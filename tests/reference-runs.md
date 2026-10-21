# Native GitHub reference evidence

The workflow `.github/workflows/example-python.yml` is an independent native reference, not WSR
execution. The pass mode must conclude success. The fail mode must conclude failure specifically
at the unittest step with the controlled assertion. Setup/network failures do not count.

Evidence will record source revision, mode, interpreter, run URL and observed conclusion after
manual dispatch. No reference execution is claimed until those runs finish and logs are checked.
