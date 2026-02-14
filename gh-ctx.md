| Workflow key                                       | Context                                                                           | Special functions                                |
| -------------------------------------------------- | --------------------------------------------------------------------------------- | ------------------------------------------------ |
| `run-name`                                         | `github, inputs, vars`                                                            | None                                             |
| `concurrency`                                      | `github, inputs, vars`                                                            | None                                             |
| `env`                                              | `github, secrets, inputs, vars`                                                   | None                                             |
| `jobs.<job_id>.concurrency`                        | `github, needs, strategy, matrix, inputs, vars`                                   | None                                             |
| `jobs.<job_id>.container`                          | `github, needs, strategy, matrix, vars, inputs`                                   | None                                             |
| `jobs.<job_id>.container.credentials`              | `github, needs, strategy, matrix, env, vars, secrets, inputs`                     | None                                             |
| `jobs.<job_id>.container.env.<env_id>`             | `github, needs, strategy, matrix, job, runner, env, vars, secrets, inputs`        | None                                             |
| `jobs.<job_id>.container.image`                    | `github, needs, strategy, matrix, vars, inputs`                                   | None                                             |
| `jobs.<job_id>.continue-on-error`                  | `github, needs, strategy, vars, matrix, inputs`                                   | None                                             |
| `jobs.<job_id>.defaults.run`                       | `github, needs, strategy, matrix, env, vars, inputs`                              | None                                             |
| `jobs.<job_id>.env`                                | `github, needs, strategy, matrix, vars, secrets, inputs`                          | None                                             |
| `jobs.<job_id>.environment`                        | `github, needs, strategy, matrix, vars, inputs`                                   | None                                             |
| `jobs.<job_id>.environment.url`                    | `github, needs, strategy, matrix, job, runner, env, vars, steps, inputs`          | None                                             |
| `jobs.<job_id>.if`                                 | `github, needs, vars, inputs`                                                     | `always, cancelled, success, failure`            |
| `jobs.<job_id>.name`                               | `github, needs, strategy, matrix, vars, inputs`                                   | None                                             |
| `jobs.<job_id>.outputs.<output_id>`                | `github, needs, strategy, matrix, job, runner, env, vars, secrets, steps, inputs` | None                                             |
| `jobs.<job_id>.runs-on`                            | `github, needs, strategy, matrix, vars, inputs`                                   | None                                             |
| `jobs.<job_id>.secrets.<secrets_id>`               | `github, needs, strategy, matrix, secrets, inputs, vars`                          | None                                             |
| `jobs.<job_id>.services`                           | `github, needs, strategy, matrix, vars, inputs`                                   | None                                             |
| `jobs.<job_id>.services.<service_id>.credentials`  | `github, needs, strategy, matrix, env, vars, secrets, inputs`                     | None                                             |
| `jobs.<job_id>.services.<service_id>.env.<env_id>` | `github, needs, strategy, matrix, job, runner, env, vars, secrets, inputs`        | None                                             |
| `jobs.<job_id>.steps.continue-on-error`            | `github, needs, strategy, matrix, job, runner, env, vars, secrets, steps, inputs` | `hashFiles`                                      |
| `jobs.<job_id>.steps.env`                          | `github, needs, strategy, matrix, job, runner, env, vars, secrets, steps, inputs` | `hashFiles`                                      |
| `jobs.<job_id>.steps.if`                           | `github, needs, strategy, matrix, job, runner, env, vars, steps, inputs`          | `always, cancelled, success, failure, hashFiles` |
| `jobs.<job_id>.steps.name`                         | `github, needs, strategy, matrix, job, runner, env, vars, secrets, steps, inputs` | `hashFiles`                                      |
| `jobs.<job_id>.steps.run`                          | `github, needs, strategy, matrix, job, runner, env, vars, secrets, steps, inputs` | `hashFiles`                                      |
| `jobs.<job_id>.steps.timeout-minutes`              | `github, needs, strategy, matrix, job, runner, env, vars, secrets, steps, inputs` | `hashFiles`                                      |
| `jobs.<job_id>.steps.with`                         | `github, needs, strategy, matrix, job, runner, env, vars, secrets, steps, inputs` | `hashFiles`                                      |
| `jobs.<job_id>.steps.working-directory`            | `github, needs, strategy, matrix, job, runner, env, vars, secrets, steps, inputs` | `hashFiles`                                      |
| `jobs.<job_id>.strategy`                           | `github, needs, vars, inputs`                                                     | None                                             |
| `jobs.<job_id>.timeout-minutes`                    | `github, needs, strategy, matrix, vars, inputs`                                   | None                                             |
| `jobs.<job_id>.with.<with_id>`                     | `github, needs, strategy, matrix, inputs, vars`                                   | None                                             |
| `on.workflow_call.inputs.<inputs_id>.default`      | `github, inputs, vars`                                                            | None                                             |
| `on.workflow_call.outputs.<output_id>.value`       | `github, jobs, vars, inputs`                                                      | None                                             |
