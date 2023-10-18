<div align="center">
  <br />
  <br />
  

  <br />
  <br />
</div>

# Opal Connector

This is the data collector that gets your system's state and sends it to the backend, it can also be used as a pure system stat inspector without needing to connect it to Opal

# ⚡ Installation

1. Go on Opal and click the + button and copy the generated token
2. Run the installation script for your platform as noted below

## 🐧 Linux

```bash
curl https://raw.githubusercontent.com/A-T-O-M-I-X/opal-connector/main/scripts/install.sh | sudo bash
```

## 🏢 Windows

### Scoop

0. Set execution policies so you can install scoop
```powershell
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
```

1. Install scoop (if you haven't already)

```powershell
iwr -useb get.scoop.sh | iex
```

2. Install Opal Connector (with admin for no popups)

```powershell
scoop install "https://raw.githubusercontent.com/A-T-O-M-I-X/opal-connector/main/scripts/opal-connector.json"
```

## 🌐 OpenWRT

This script updates an existing installation of Opal Connector. It will not work if you have not already installed Opal Connector.
```bash
wget https://raw.githubusercontent.com/A-T-O-M-I-X/opal-connector/main/scripts/update-mipsel.sh -O /tmp/update-mipsel.sh && chmod +x /tmp/update-mipsel.sh && /tmp/update-mipsel.sh && rm /tmp/update-mipsel.sh
```
