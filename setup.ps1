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
    Write-Host "Node.js is not installed. Please install Node.js and try again." -ForegroundColor Red
    exit 1
}

# Check for Rust and Cargo
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Rust is not installed. Installing Rust..." -ForegroundColor Yellow
    Invoke-Expression (iwr -useb https://sh.rustup.rs | Invoke-Expression)
}

# Step 2: Clone the repository
Write-Host "Cloning the repository..."
if (-not (Test-Path "2")) {
    git clone https://github.com/your-repo-url.git 2
    cd 2
} else {
    Write-Host "Repository already exists. Skipping clone step."
    cd 2
}

# Step 3: Install Node.js dependencies
Write-Host "Installing Node.js dependencies..."
npm install

# Step 4: Install Rust dependencies
Write-Host "Installing Rust dependencies..."
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