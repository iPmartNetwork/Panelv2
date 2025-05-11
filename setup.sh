#!/bin/bash

# Bash Script for Automatic Installation and Setup

# Step 1: Check for required tools
echo "Checking for required tools..."

# Check for Git
if ! command -v git &> /dev/null
then
    echo "Git is not installed. Please install Git and try again."
    exit 1
fi

# Check for Node.js and npm
if ! command -v node &> /dev/null
then
    echo "Node.js is not installed. Please install Node.js and try again."
    exit 1
fi

# Check for Rust and Cargo
if ! command -v cargo &> /dev/null
then
    echo "Rust is not installed. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Step 2: Clone the repository
echo "Cloning the repository..."
if [ ! -d "Panelv2" ]; then
    git clone https://github.com/iPmartNetwork/Panelv2.git
    cd Panelv2
else
    echo "Repository already exists. Skipping clone step."
    cd Panelv2
fi

# Step 3: Install Node.js dependencies
echo "Installing Node.js dependencies..."
npm install

# Step 4: Install Rust dependencies
echo "Installing Rust dependencies..."
cargo build --release

# Step 5: Run the application
echo "Starting the application..."
npm run dev &

# Step 6: Create a systemd service for auto-start (Linux only)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "Creating a systemd service to auto-start the application after reboot..."
    SERVICE_PATH="/etc/systemd/system/panelv2.service"
    echo "[Unit]" > panelv2.service
    echo "Description=Panelv2 Service" >> panelv2.service
    echo "After=network.target" >> panelv2.service

    echo "[Service]" >> panelv2.service
    echo "Type=simple" >> panelv2.service
    echo "ExecStart=$(pwd)/start.sh" >> panelv2.service
    echo "Restart=always" >> panelv2.service

    echo "[Install]" >> panelv2.service
    echo "WantedBy=multi-user.target" >> panelv2.service

    sudo mv panelv2.service $SERVICE_PATH
    sudo systemctl daemon-reload
    sudo systemctl enable panelv2
    sudo systemctl start panelv2
    echo "Systemd service created successfully!"
else
    echo "Systemd services are not supported on this OS. Skipping service creation."
fi

echo "Setup complete! The application is running and will auto-start after reboot (if supported)."