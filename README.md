# pbinfo-cli

This project is a command line interface for pbinfo. It is written in rust and uses reqwest for making gets and posts. 

**VERY IMPORTANT:** As of now this project uses plain text to store your pbinfo username and password so be very carefull of using this. This will be changed in the near future.

## Building and running

Right now the only way of running this is building the projects yourself, thankfully this is pretty easy.

### Dependencies
All you need is rust, openssl, pkg-config

- **Nix:**
All you need to do is run `nix-shell` in the repo and everything will be installed

- **Ubuntu:**
```
sudo apt install -y cargo libssl-dev pkg-config
``` 

- **Fedora:**
```
sudo yum install pkg-config openssl-devel
```

- **Windows:**
I will be using [chocolatey](https://chocolatey.org/) for this, [here](https://chocolatey.org/install) is a very simple tutorial on installing it.

After you have installed it just run in an admin cmd or ps:
```
choco install rust openssl pkgconfiglite
```

After we installed these we just need to build it
### Building
```
cargo build
```
the executable will be located under `target/debug/pbinfo-cli(.exe)`

## General usage
The way you use it is go into a project, run the executable using the `--problem-id` flag to specify the id of the problem. After running the program pbinfo-cli will look for a file named main.cpp in your folder and send it to pbinfo.

