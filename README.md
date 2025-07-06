
<h1 align="center">
  HT - WIP
  <br>
</h1>

<h4 align="center">A simple CLI tool to support and automate deployments for Salesforce.</h4>

<p align="center">
  <a href="#key-features">Key Features</a> •
  <a href="#how-to-use">How To Use</a> •
</p>

## Key Features

* `ht verify`
  - Builds your sfdx source project to a sandbox of your choosing.
    - Handles running of pre- and post-deployment anonymous apex scripts
    - Installs dependendent packages 
* `ht version`
  - Creates a new version of your package. Requires [conventional commit](https://www.conventionalcommits.org/en/v1.0.0/) format to generate the next version number.
    - Option to tag and/or create a commit with the new package version

## How To Use

To install this application, ensure that the Rust toolchain is installed in your system. Then, clone this repo and run `cargo run`.

