#!/bin/bash

# Bash Script for Automatic Installation and Setup for Panelv2

# Exit immediately if a command exits with a non-zero status.
set -e

# --- Configuration ---
GIT_REPO_URL="https://github.com/iPmartNetwork/Panelv2.git"
PROJECT_DIR_NAME="Panelv2"
# By default, clone into a subdirectory where the script is run.
# If you run this script via curl | bash, it will be in the current directory of that command.
INSTALL_BASE_DIR=$(pwd)
PROJECT_FULL_PATH="$INSTALL_BASE_DIR/$PROJECT_DIR_NAME"
START_SCRIPT_NAME="start_panel.sh" # Renamed to avoid conflict if user has start.sh
SERVICE_NAME="panelv2"

# --- Helper Functions ---
print_info() {
    echo -e "\\033[1;34m[INFO]\\033[0m $1"
}

print_success() {
    echo -e "\\033[1;32m[SUCCESS]\\033[0m $1"
}

print_warning() {
    echo -e "\\033[1;33m[WARNING]\\033[0m $1"
}

print_error() {
    echo -e "\\033[1;31m[ERROR]\\033[0m $1" >&2
}

# --- Prerequisite Installation ---
install_prerequisites() {
    print_info "Checking and installing prerequisites..."

    # Install Git
    if ! command -v git &> /dev/null; then
        print_info "Git not found. Installing Git..."
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            sudo apt-get update && sudo apt-get install -y git
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            brew install git
        else
            print_error "Unsupported OS for automatic Git installation. Please install Git manually."
            exit 1
        fi
        print_success "Git installed."
    else
        print_info "Git is already installed."
    fi

    # Install Node.js (LTS) and npm
    if ! command -v node &> /dev/null || ! command -v npm &> /dev/null; then
        print_info "Node.js or npm not found. Installing Node.js (LTS)..."
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
            sudo apt-get install -y nodejs
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            brew install node
        else
            print_error "Unsupported OS for automatic Node.js installation. Please install Node.js (LTS) and npm manually."
            exit 1
        fi
        print_success "Node.js and npm installed."
    else
        print_info "Node.js and npm are already installed."
    fi

    # Install build tools (C compiler/linker) - Essential for some Rust crates
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v apt-get &> /dev/null; then # Debian/Ubuntu based
            if ! dpkg -s build-essential &> /dev/null; then
                print_info "build-essential not found. Installing build-essential..."
                sudo apt-get update && sudo apt-get install -y build-essential
                print_success "build-essential installed."
            else
                print_info "build-essential is already installed."
            fi
        elif command -v dnf &> /dev/null; then # Fedora based
            if ! dnf group info "Development Tools" | grep -q "Installed Groups"; then
                print_info "Development Tools group not found. Installing Development Tools (includes gcc, make)..."
                sudo dnf groupinstall -y "Development Tools"
                print_success "Development Tools installed."
            else
                print_info "Development Tools group is already installed."
            fi
        elif command -v yum &> /dev/null; then # Older CentOS/RHEL
             if ! yum groupinfo "Development Tools" | grep -q "Installed Groups"; then
                print_info "Development Tools group not found. Installing Development Tools (includes gcc, make)..."
                sudo yum groupinstall -y "Development Tools"
                print_success "Development Tools installed."
            else
                print_info "Development Tools group is already installed."
            fi
        elif command -v pacman &> /dev/null; then # Arch based
            if ! pacman -Q base-devel &> /dev/null; then
                print_info "base-devel group not found. Installing base-devel..."
                sudo pacman -S --noconfirm base-devel
                print_success "base-devel installed."
            else
                print_info "base-devel group is already installed."
            fi
        else
            print_warning "Could not determine package manager for installing build tools. Please ensure a C compiler (like gcc) and make are installed."
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        # On macOS, Xcode Command Line Tools provide the C compiler (clang)
        if ! xcode-select -p &> /dev/null; then
            print_info "Xcode Command Line Tools not found. Installing..."
            xcode-select --install
            # This command opens a dialog. User interaction is required.
            # Script might need to pause or user run this separately.
            print_warning "Please follow the on-screen instructions to install Xcode Command Line Tools. Re-run this script after installation is complete."
            # exit 1 # Optionally exit and ask user to re-run
        else
            print_info "Xcode Command Line Tools are already installed."
        fi
    fi

    # Install Rust and Cargo
    if ! command -v cargo &> /dev/null; then
        print_info "Rust (cargo) not found. Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # Source cargo environment for the current script session
        if [ -f "$HOME/.cargo/env" ]; then
            source "$HOME/.cargo/env"
        else
            print_warning "Could not source cargo env. Cargo might not be in PATH for subsequent steps in this script if rustup didn't update PATH for the current session."
        fi
        print_success "Rust and Cargo installed."
        # Add the required target for cross-compilation
        print_info "Adding armv7-unknown-linux-gnueabihf target for Rust..."
        rustup target add armv7-unknown-linux-gnueabihf
        print_success "armv7-unknown-linux-gnueabihf target added."
    else
        print_info "Rust (cargo) is already installed."
        # Ensure cargo env is sourced if script is re-run and cargo was just installed
        if [ -f "$HOME/.cargo/env" ] && ! command -v cargo &>/dev/null; then
             source "$HOME/.cargo/env"
        fi
        # Ensure the target is present even if Rust was already installed
        if ! rustup target list --installed | grep -q "armv7-unknown-linux-gnueabihf"; then
            print_info "Adding armv7-unknown-linux-gnueabihf target for Rust..."
            rustup target add armv7-unknown-linux-gnueabihf
            print_success "armv7-unknown-linux-gnueabihf target added."
        else
            print_info "armv7-unknown-linux-gnueabihf target is already installed."
        fi
    fi
}

# --- Project Setup ---
setup_project() {
    print_info "Setting up Panelv2 project..."

    # Clone repository
    if [ ! -d "$PROJECT_FULL_PATH" ]; then
        print_info "Cloning repository from $GIT_REPO_URL into $PROJECT_FULL_PATH..."
        git clone "$GIT_REPO_URL" "$PROJECT_FULL_PATH"
        print_success "Repository cloned."
    else
        print_info "Project directory $PROJECT_FULL_PATH already exists. Skipping clone."
        print_info "Attempting to update the existing repository..."
        cd "$PROJECT_FULL_PATH"
        git pull || print_warning "git pull failed. Continuing with existing local version."
        cd "$INSTALL_BASE_DIR" # Go back to original directory
    fi

    cd "$PROJECT_FULL_PATH"

    # Install Node.js dependencies
    print_info "Installing Node.js dependencies..."
    npm install
    print_success "Node.js dependencies installed."

    # Build Rust project
    print_info "Building Rust project (release mode)..."
    # Ensure cargo is available for this step
    if ! command -v cargo &> /dev/null; then
        if [ -f "$HOME/.cargo/env" ]; then
            source "$HOME/.cargo/env"
        fi
        if ! command -v cargo &> /dev/null; then
            print_error "Cargo command not found even after attempting to source env. Cannot build Rust project."
            exit 1
        fi
    fi
    cargo build --release
    print_success "Rust project built."

    cd "$INSTALL_BASE_DIR" # Go back to original directory
}

# --- Create Start Script ---
create_start_script() {
    print_info "Creating start script ($START_SCRIPT_NAME) at $PROJECT_FULL_PATH/$START_SCRIPT_NAME..."
    
    # Determine Node.js path (important if nvm is used, though script installs system-wide for Linux)
    NODE_EXECUTABLE=$(command -v node || echo "node")
    NPM_EXECUTABLE=$(command -v npm || echo "npm")

    cat << EOF > "$PROJECT_FULL_PATH/$START_SCRIPT_NAME"
#!/bin/bash
set -e

echo "Starting Panelv2..."
cd "$PROJECT_FULL_PATH"

# Ensure Rust/Cargo binaries are in PATH
# This is crucial if the script is run by systemd or cron with a minimal environment
if [ -d "\$HOME/.cargo/bin" ] && [[ ":\$PATH:" != *":\$HOME/.cargo/bin:"* ]]; then
    export PATH="\$HOME/.cargo/bin:\$PATH"
fi

# If Node.js was installed via NVM, NVM might need to be sourced.
# This script assumes Node/NPM are in PATH (e.g. system install or user's .bashrc/.profile handled it)
# For systemd, the service file should handle PATH or User environment.

echo "Using Node: $NODE_EXECUTABLE"
echo "Using npm: $NPM_EXECUTABLE"
echo "Current PATH: \$PATH"
echo "Running 'npm run dev' in $PROJECT_FULL_PATH..."

"$NPM_EXECUTABLE" run dev
EOF

    chmod +x "$PROJECT_FULL_PATH/$START_SCRIPT_NAME"
    print_success "Start script created and made executable."
}

# --- Systemd Service Setup (Linux Only) ---
setup_systemd_service() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        print_info "Setting up systemd service ($SERVICE_NAME)..."
        
        RUN_USER=$(whoami)
        USER_HOME=$(getent passwd "$RUN_USER" | cut -d: -f6)
        
        # Construct PATH for systemd service
        # Includes standard paths and user's cargo bin
        SERVICE_PATH_ENV="$USER_HOME/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
        # If Node was installed via NVM, its path might need to be added too.
        # For NodeSource, /usr/bin/node should be covered.

        SERVICE_FILE_PATH="/etc/systemd/system/${SERVICE_NAME}.service"

        print_info "Creating systemd service file at $SERVICE_FILE_PATH..."
        
        # Use sudo tee to write the service file
        # Using a temporary file for sudo tee to handle multi-line content easily
        TEMP_SERVICE_FILE=$(mktemp)
        cat << EOF > "$TEMP_SERVICE_FILE"
[Unit]
Description=Panelv2 Service
After=network.target

[Service]
Type=simple
User=$RUN_USER
Group=$(id -g -n "$RUN_USER")
WorkingDirectory=$PROJECT_FULL_PATH
ExecStart=$PROJECT_FULL_PATH/$START_SCRIPT_NAME
Restart=always
RestartSec=10
Environment="PATH=$SERVICE_PATH_ENV"
# Add other environment variables if needed, e.g., for Node.js if installed via NVM
# Environment="NVM_DIR=$USER_HOME/.nvm"
# ExecStart=/bin/bash -c 'source \$NVM_DIR/nvm.sh && $PROJECT_FULL_PATH/$START_SCRIPT_NAME' # If NVM

StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

        sudo cp "$TEMP_SERVICE_FILE" "$SERVICE_FILE_PATH"
        rm "$TEMP_SERVICE_FILE"
        sudo chmod 644 "$SERVICE_FILE_PATH"

        print_info "Reloading systemd daemon, enabling and starting ${SERVICE_NAME} service..."
        sudo systemctl daemon-reload
        sudo systemctl enable "${SERVICE_NAME}.service"
        sudo systemctl start "${SERVICE_NAME}.service"
        
        print_success "Systemd service ${SERVICE_NAME} created and started."
        print_info "You can check the status with: sudo systemctl status ${SERVICE_NAME}"
        print_info "And view logs with: sudo journalctl -u ${SERVICE_NAME} -f"
    else
        print_warning "Systemd service setup is skipped as OS is not Linux."
        print_info "You can start the application manually by running: $PROJECT_FULL_PATH/$START_SCRIPT_NAME"
    fi
}

# --- Main Execution ---
main() {
    print_info "Panelv2 Automatic Setup Script"
    install_prerequisites
    setup_project
    create_start_script
    setup_systemd_service
    print_success "Panelv2 setup finished!"
    if [[ "$OSTYPE" != "linux-gnu"* ]]; then
         print_info "To start the panel, run: $PROJECT_FULL_PATH/$START_SCRIPT_NAME"
    fi
}

# Run the main function
main
