use anyhow::{bail, Context, Result};
use std::{
  collections::HashMap,
  env,
  io::{self, Write},
  path::PathBuf,
  process::Command,
};

mod terraform;

fn main() -> Result<()> {
  let mut args = env::args().skip(1);

  let root_module_path = args
    .next()
    .map(|p_str| PathBuf::from(p_str))
    .unwrap_or(env::current_dir().context("Failed to get current working dir")?);

  // TODO: read in modules dir (structopt?)

  // TODO: ensure tf fmt and tf validate?

  // get list of resources in state

  let tsl_output = Command::new("terraform")
    .current_dir(&root_module_path)
    .arg("state")
    .arg("list")
    .output()
    .context("Failed to run `terraform state list`")?;

  if !tsl_output.status.success() {
    io::stderr().write_all(&tsl_output.stderr)?;
    bail!("`terraform state list` failed with status code {}", tsl_output.status);
  }

  let state_res = String::from_utf8(tsl_output.stdout).context("Failed to parse `terraform state list` output")?;

  // collect modules

  let root_module = terraform::Module::from_path(&root_module_path)?;

  // for local modules, compare resources

  let (total, unused) = state_res
    .lines()
    .filter_map(|line| {
      if line.starts_with("module.") {
        let mut parts = line.split('.').skip(1);
        let module_name = parts.next().unwrap();
        let mut is_data = false;
        let typ = {
          let typ_or_data = parts.next().unwrap();
          if typ_or_data == "data" {
            is_data = true;
            parts.next().unwrap()
          } else {
            typ_or_data
          }
        }
        .to_string();
        let name = {
          let n = parts.next().unwrap();
          n.split_once('[').map_or(n, |x| x.0)
        }
        .to_string();
        Some((line, module_name, is_data, typ, name))
      } else {
        None
      }
    })
    .fold(HashMap::new(), |mut acc, (line, module_name, is_data, typ, name)| {
      let mb = root_module.modules.get(module_name).unwrap();
      let m = acc
        .entry(mb.source_path.clone())
        .or_insert(terraform::Module::from_path(&mb.source_path).unwrap());

      if is_data {
        if let Some(dsb) = m.data_sources.get_mut(&(typ, name)) {
          dsb.used = true;
        } else {
          eprintln!(
            "ERROR: {} in state but not found in code, drifted deployment or report bug",
            line
          );
        }
      } else {
        if let Some(rb) = m.resources.get_mut(&(typ, name)) {
          rb.used = true;
        } else {
          eprintln!(
            "ERROR: {} in state but not found in code, drifted deployment or report bug",
            line
          );
        }
      }

      acc
    })
    .values()
    .fold((0, 0), |(mut total, mut unused), m| {
      m.data_sources.values().chain(m.resources.values()).for_each(|res| {
        total += res.lineno.1 - res.lineno.0 + 1;
        if !res.used {
          println!("{} {} - {}", res.file.to_str().unwrap(), res.lineno.0, res.lineno.1);
          unused += res.lineno.1 - res.lineno.0 + 1;
        }
      });

      (total, unused)
    });

  println!("Checked LoC: {}", total);
  println!("Unused LoC: {}", unused);
  println!("Coverage: {:.2}%", ((total - unused) as f64 / total as f64) * 100.0);

  // TODO: generate coverage report in recognizable format?

  Ok(())
}
