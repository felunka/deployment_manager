# Deployment manager

---

#### ⚠️ Warning
This is still under development and there are some known security limitations in the current implementation! Use with care!

Most importantly: Installing the server agent will give the web service the ability to use sudo to install the GitHub actions runner service. This means, if a attacker finds a way to modify the service install script, he can execute any code as root. Also, web server inputs are not validated enough right now.

---

This tool is meant to help you create, manage, monitor and back up your fun side projects you host on vservers. It supports connecting to multiple servers (nodes) and is mostly focussed on web applications. The installation will make use of a traefik reverse proxy on every node with Let's Encrypt certbot setup.

## Pattern

## Installing

### First install (chicken and egg problem)

You want to host this tool and manage it later by itself? Here is how to do it:

1. Install the agent on your server. Make sure to replace both the hostname and Email. The Email is used for Let's Encrypt certs:
```
curl -fsSL https://github.com/felunka/deployment_manager/raw/refs/heads/main/public/install.sh | sudo bash -s -- "https://github.com/felunka/deployment_manager/raw/refs/heads/main/public" NODE_HOSTNAME ACME_EMAIL
```


2. Clone the GitHub repo and run the compose file
```
cd /home/node_agent
git clone git@github.com:felunka/deployment_manager.git
cd deployment_manager
RAILS_MASTER_KEY=CHOOSE POSTGRES_PASSWORD=CHOOSE DOMAIN=manager.example docker compose up -d
```
OR

Fork the repository and setup your self hosted runner. You can then use the GitHub workflow file for deployment.

3. Create yourself a user account:
Connect to a docker container shell, open a rails shell and create a user account.

4. Login and first create the node and then a deployment using the adopt option

### New nodes

Once you have the tool running, adding a new node is much simple. Just go on the node create page and use the provided command for automatic setup
