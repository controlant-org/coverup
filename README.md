# Coverup

An interpretation of "test coverage" for Terraform.

## Measurements

This tool currently checks [data sources](https://www.terraform.io/language/data-sources) and [resources](https://www.terraform.io/language/resources/syntax) defined in Terraform code bases, against what is actually deployed - which it gets by reading a deployment's [state](https://www.terraform.io/language/state).

It currently operates on block level, not line level - so for example it does not check [dynamic blocks](https://www.terraform.io/language/expressions/dynamic-blocks) nor field level [conditionals](https://www.terraform.io/language/expressions/conditionals).

## Rationale

Despite questionable usefulness, "test coverage" remains a - perhaps the most - popular metrics to measure code "quality". Terraform code, as an implementation of "Infrastructure as Code" (IaC), can be argued to require the same metrics. However, it is perhaps not as clear as "functionality code" in terms of how to gauge it, or even what should we be testing.

Usually, "tests" are performed against the results of _execution_ of the "code". Our argument follows that, the coverage of Terraform code should be from `terraform apply` execution results - which is recorded in the state file. This is why merely using `terraform validate` is not enough, as it does not actually execute the code (providers, actually) and unfortunately the syntax alone is not enough to cover all edge cases, and actual target systems can enforce additional constraints when creating or updating resources.

Note we're not testing, for example "provisioning this AWS resource should call this AWS API with this set of parameters". These lower level details are covered by the [comprehensive set of tests](https://github.com/hashicorp/terraform-provider-aws/blob/main/docs/contributing/running-and-writing-acceptance-tests.md)  Terraform itself and the providers provide.

Nowadays, most if not all services will be deployed to various internal development or testing environments before being release to external customers. IaC is a powerful tool in this respect, not only to capture the shared basis between all deployments on various environments, but also those little (or big) differences between environments and expose them directly in code.

And this angle leads us to the interpretation that, the "test" is - at least in non-production environments - somewhat an integration (or e2e) test in the form of running `terraform apply` successfully. And the "coverage" is how much of the code is actually deployed. The environmental differences, and how much of the Terraform code are not "tested" before applying them in production environments, can thus be quantified by this tool.
