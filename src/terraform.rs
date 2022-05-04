use anyhow::{bail, Context, Result};
use std::{collections::HashMap, ffi::OsStr, fs, path::PathBuf};

pub type LineNo = (usize, usize);
pub type TypName = (String, String);

#[derive(Debug, Clone)]
pub struct Module {
  path: PathBuf,
  files: Vec<PathBuf>,
  lines_total: usize,

  pub modules: HashMap<String, ModuleBlock>,
  pub data_sources: HashMap<TypName, ResBlock>,
  pub resources: HashMap<TypName, ResBlock>,
}

#[derive(Debug, Clone)]
pub struct ModuleBlock {
  pub source_path: PathBuf,
  pub lineno: LineNo,
  pub used: bool,
}

#[derive(Debug, Clone)]
pub struct ResBlock {
  pub file: PathBuf,
  pub lineno: LineNo,
  pub used: bool,
}

impl Module {
  // MAYBE: use io::Error
  pub fn from_path(path: &PathBuf) -> Result<Self> {
    let files: Vec<PathBuf> = fs::read_dir(path)
      .with_context(|| format!("Failed to read directory {:?}", path))?
      .filter_map(|r| r.ok())
      .map(|e| e.path())
      .filter(|p| p.is_file() && p.extension().map_or(false, |ext| ext == OsStr::new("tf")))
      .collect();

    let mut lines_total = 0;
    let mut modules = HashMap::new();
    let mut data_sources = HashMap::new();
    let mut resources = HashMap::new();

    enum Current {
      None,
      Module(String, ModuleBlock),
      DataSource(TypName, ResBlock),
      Resource(TypName, ResBlock),
    }

    let mut cur = Current::None;

    for f in files.iter() {
      let content = fs::read_to_string(&f).with_context(|| format!("Failed to read file {:?}", f))?;
      for (ln0, line) in content.lines().enumerate() {
        if line.trim_end().is_empty() {
          continue;
        }
        lines_total += 1;
        let ln = ln0 + 1;

        if line.trim_end() == "}" {
          match cur {
            Current::None => {}
            Current::Module(name, ref mut m) => {
              m.lineno.1 = ln;
              modules.insert(name, m.clone());
            }
            Current::DataSource(tn, ref mut r) => {
              r.lineno.1 = ln;
              data_sources.insert(tn, r.clone());
            }
            Current::Resource(tn, ref mut r) => {
              r.lineno.1 = ln;
              resources.insert(tn, r.clone());
            }
          }
          cur = Current::None;
        }

        if let Current::Module(_, ref mut m) = cur {
          if line.trim().starts_with("source") {
            let p = line
              .split_once('=')
              .context("Failed to parse source path")?
              .1
              .trim()
              .trim_matches('"');

            m.source_path.pop();

            m.source_path.push(p);

            // TODO: handle non-local modules

            m.source_path = m
              .source_path
              .canonicalize()
              .with_context(|| format!("Failed to canonicalize {:?}", m.source_path))?;
          }
        }

        if line.starts_with("module ") {
          let name = line
            .split_at(8)
            .1
            .split('"')
            .next()
            .with_context(|| format!("Failed to parse module name at line {}", ln))?
            .to_string();

          cur = Current::Module(
            name,
            ModuleBlock {
              source_path: f.clone(),
              lineno: (ln, 0),
              used: false,
            },
          );
        }

        if line.starts_with("data ") {
          let mut parts = line.split("\" \"");
          let typ = parts
            .next()
            .with_context(|| format!("Failed to parse data source type at line {}", ln))?
            .split_at(6)
            .1
            .to_string();
          let name = parts
            .next()
            .with_context(|| format!("Failed to parse data source name at line {}", ln))?
            .split("\" ")
            .next()
            .with_context(|| format!("Failed to parse data source name at line {}", ln))?
            .to_string();

          if line.trim_end().ends_with("}") {
            data_sources.insert(
              (typ, name),
              ResBlock {
                file: f.clone(),
                lineno: (ln, ln),
                used: false,
              },
            );
          } else {
            cur = Current::DataSource(
              (typ, name),
              ResBlock {
                file: f.clone(),
                lineno: (ln, 0),
                used: false,
              },
            );
          }
        }

        if line.starts_with("resource ") {
          let mut parts = line.split("\" \"");
          let typ = parts
            .next()
            .with_context(|| format!("Failed to parse resource type at line {}", ln))?
            .split_at(10)
            .1
            .to_string();
          let name = parts
            .next()
            .with_context(|| format!("Failed to parse resource name at line {}", ln))?
            .split("\" ")
            .next()
            .with_context(|| format!("Failed to parse resource name at line {}", ln))?
            .to_string();

          if line.trim_end().ends_with("}") {
            resources.insert(
              (typ, name),
              ResBlock {
                file: f.clone(),
                lineno: (ln, ln),
                used: false,
              },
            );
          } else {
            cur = Current::Resource(
              (typ, name),
              ResBlock {
                file: f.clone(),
                lineno: (ln, 0),
                used: false,
              },
            );
          }
        }
      }
    }

    Ok(Self {
      path: path.clone(),
      files,
      lines_total,
      modules,
      data_sources,
      resources,
    })
  }
}
