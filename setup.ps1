# PowerShell Script for Automatic Installation and Setup

# Step 1: Check for required tools
Write-Host "Checking for required tools..."

# Check for Git
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host "Git is not installed. Please install Git and try again." -ForegroundColor Red
    exit 1
}

# Check for Node.js and npm
if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "Node.js is not installed. Installing Node.js using winget..." -ForegroundColor Yellow
    winget install OpenJS.NodeJS.LTS -e --accept-package-agreements --accept-source-agreements
    if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
        Write-Host "Node.js installation failed. Please install Node.js manually and re-run the script." -ForegroundColor Red
        exit 1
    }
}

# Check for Rust and Cargo
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Rust is not installed. Installing Rust..." -ForegroundColor Yellow
    Invoke-Expression (New-Object System.Net.WebClient).DownloadString('https://sh.rustup.rs') | Invoke-Expression -ArgumentList '-y --default-toolchain stable-msvc'
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
         Write-Host "Rust installation failed or PATH not updated. Please install Rust manually (https://www.rust-lang.org/tools/install) with MSVC toolchain and re-run the script, or open a new PowerShell window." -ForegroundColor Red
        exit 1
    }
}

# Check for MSVC build tools (linker)
$rustcVersion = (rustc -Vv | Select-String "host:")
if ($rustcVersion -match "msvc") {
    Write-Host "MSVC toolchain detected."
    # Attempt to run a simple C compile test or check for linker directly if possible
    # For now, we assume if rustup installed msvc toolchain, it *should* work.
    # A more robust check would be to try and compile a dummy C file.
} else {
    Write-Host "MSVC toolchain not detected or not default. Attempting to install/set it up..." -ForegroundColor Yellow
    rustup toolchain install stable-msvc
    rustup default stable-msvc
    # Re-check after attempting to install
    $rustcVersionAfterInstall = (rustc -Vv | Select-String "host:")
    if ($rustcVersionAfterInstall -match "msvc") {
        Write-Host "MSVC toolchain installed and set as default." -ForegroundColor Green
    } else {
        Write-Host "Failed to set up MSVC toolchain automatically. Please ensure Visual Studio Build Tools with C++ are installed, or run 'rustup toolchain install stable-msvc' and 'rustup default stable-msvc' manually." -ForegroundColor Red
        Write-Host "You might need to install 'Desktop development with C++' workload from Visual Studio Installer." -ForegroundColor Yellow
        exit 1
    }
}

# Step 2: Clone the repository
Write-Host "Cloning the repository..."
if (-not (Test-Path "2")) {
    git clone https://github.com/iPmartNetwork/Panelv2.git Panelv2
    cd Panelv2
} else {
    Write-Host "Repository already exists. Skipping clone step."
    cd Panelv2
}

# Step 3: Install Node.js dependencies
Write-Host "Installing Node.js dependencies..."
npm install

# Step 4: Install Rust dependencies
Write-Host "Installing Rust dependencies..."
# Ensure Cargo is available in the current session's PATH
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
cargo build --release

# Step 5: Run the application
Write-Host "Starting the application..."
Start-Process -NoNewWindow -FilePath "cmd.exe" -ArgumentList "/c", "npm run dev" -WorkingDirectory (Get-Location)

Write-Host "Setup complete! The application is running." -ForegroundColor Green

# Step 6: Create a Windows Service for Auto-Start
Write-Host "Creating a Windows Service to auto-start the application after reboot..."

# Define service parameters
$serviceName = "MyAppService"
$serviceDescription = "Service to auto-start the application after reboot."
$servicePath = "$(Get-Location)\start_app.bat"

# Create a batch file to start the application
Write-Host "Creating a batch file to start the application..."
$batchContent = "cd /d $(Get-Location) && npm run dev"
Set-Content -Path "start_app.bat" -Value $batchContent

# Check if the service already exists
if (Get-Service -Name $serviceName -ErrorAction SilentlyContinue) {
    Write-Host "Service already exists. Updating the service..."
    sc.exe delete $serviceName | Out-Null
}

# Create the service
sc.exe create $serviceName binPath= "$servicePath" DisplayName= "$serviceName" start= auto | Out-Null
sc.exe description $serviceName "$serviceDescription" | Out-Null

# Start the service
Start-Service -Name $serviceName
Write-Host "Service created and started successfully!" -ForegroundColor Green