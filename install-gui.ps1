# Amni-Code GUI Installer
# Requires PowerShell with Windows Forms

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

# Create main form
$form = New-Object System.Windows.Forms.Form
$form.Text = "Amni-Code Agent Installer v0.3.0"
$form.Size = New-Object System.Drawing.Size(600, 500)
$form.StartPosition = "CenterScreen"
$form.BackColor = [System.Drawing.Color]::FromArgb(30, 30, 30)
$form.ForeColor = [System.Drawing.Color]::White

# Title label
$titleLabel = New-Object System.Windows.Forms.Label
$titleLabel.Text = "Amni-Code AI Assistant Installer"
$titleLabel.Font = New-Object System.Drawing.Font("Arial", 16, [System.Drawing.FontStyle]::Bold)
$titleLabel.Size = New-Object System.Drawing.Size(550, 40)
$titleLabel.Location = New-Object System.Drawing.Point(20, 20)
$titleLabel.TextAlign = [System.Drawing.ContentAlignment]::MiddleCenter
$form.Controls.Add($titleLabel)

# Progress bar
$progressBar = New-Object System.Windows.Forms.ProgressBar
$progressBar.Size = New-Object System.Drawing.Size(550, 30)
$progressBar.Location = New-Object System.Drawing.Point(20, 420)
$progressBar.Style = [System.Windows.Forms.ProgressBarStyle]::Continuous
$form.Controls.Add($progressBar)

# Status label
$statusLabel = New-Object System.Windows.Forms.Label
$statusLabel.Text = "Ready to install..."
$statusLabel.Size = New-Object System.Drawing.Size(550, 30)
$statusLabel.Location = New-Object System.Drawing.Point(20, 380)
$statusLabel.TextAlign = [System.Drawing.ContentAlignment]::MiddleLeft
$form.Controls.Add($statusLabel)

# Log text box
$logTextBox = New-Object System.Windows.Forms.TextBox
$logTextBox.Multiline = $true
$logTextBox.ScrollBars = "Vertical"
$logTextBox.Size = New-Object System.Drawing.Size(550, 200)
$logTextBox.Location = New-Object System.Drawing.Point(20, 160)
$logTextBox.BackColor = [System.Drawing.Color]::Black
$logTextBox.ForeColor = [System.Drawing.Color]::Green
$logTextBox.Font = New-Object System.Drawing.Font("Consolas", 9)
$form.Controls.Add($logTextBox)

# Install button
$installButton = New-Object System.Windows.Forms.Button
$installButton.Text = "Start Installation"
$installButton.Size = New-Object System.Drawing.Size(150, 40)
$installButton.Location = New-Object System.Drawing.Point(20, 70)
$installButton.BackColor = [System.Drawing.Color]::FromArgb(0, 122, 204)
$installButton.ForeColor = [System.Drawing.Color]::White
$form.Controls.Add($installButton)

# Cancel button
$cancelButton = New-Object System.Windows.Forms.Button
$cancelButton.Text = "Cancel"
$cancelButton.Size = New-Object System.Drawing.Size(100, 40)
$cancelButton.Location = New-Object System.Drawing.Point(180, 70)
$form.Controls.Add($cancelButton)

# Function to log messages
function Log-Message {
    param([string]$message)
    $timestamp = Get-Date -Format "HH:mm:ss"
    $logTextBox.AppendText("[$timestamp] $message`r`n")
    $logTextBox.ScrollToCaret()
    [System.Windows.Forms.Application]::DoEvents()
}

# Function to update progress
function Update-Progress {
    param([int]$percent, [string]$status)
    $progressBar.Value = $percent
    $statusLabel.Text = $status
    [System.Windows.Forms.Application]::DoEvents()
}

# Function to run command and log output
function Run-Command {
    param([string]$command, [string]$description)
    Log-Message "Running: $description"
    try {
        $output = Invoke-Expression $command 2>&1
        if ($LASTEXITCODE -eq 0) {
            Log-Message "OK $description completed"
            return $true
        } else {
            Log-Message "ERROR $description failed: $output"
            return $false
        }
    } catch {
        Log-Message "ERROR $description error: $($_.Exception.Message)"
        return $false
    }
}

# Function to perform installation
function Install-AmniCode {
    Update-Progress 0 "Starting installation..."

    # Step 1: Check Rust
    Update-Progress 5 "Checking Rust installation..."
    if (!(Get-Command rustc -ErrorAction SilentlyContinue)) {
        Log-Message "Rust not found. Installing..."
        $cargoBin = Join-Path (Join-Path $env:USERPROFILE ".cargo") "bin"
        $env:PATH += ";$cargoBin"
    } else {
        Log-Message "Rust is already installed"
    }

    # Step 2: Check Python
    Update-Progress 10 "Checking Python installation..."
    $pythonFound = $false
    if (Get-Command python -ErrorAction SilentlyContinue) {
        Log-Message "OK Python found via 'python'"
        $pythonFound = $true
    } elseif (Get-Command py -ErrorAction SilentlyContinue) {
        Log-Message "OK Python found via 'py'"
        $pythonFound = $true
    }

    if (!$pythonFound) {
        Log-Message "Python not found. Opening Microsoft Store..."
        Start-Process "ms-windows-store://pdp/?productid=9PJPW5LDXLZ5"
        [System.Windows.Forms.MessageBox]::Show("Please install Python 3.8+ from Microsoft Store, then click OK to continue.", "Python Installation Required", [System.Windows.Forms.MessageBoxButtons]::OK, [System.Windows.Forms.MessageBoxIcon]::Information)
    }

    # Step 3: Hardware detection
    Update-Progress 15 "Detecting hardware..."
    $hasNvidia = $false
    $hasAmd = $false
    $hasCuda = $false
    $hasHip = $false

    # Check NVIDIA
    try {
        $nvidiaOutput = nvidia-smi 2>$null
        if ($LASTEXITCODE -eq 0) {
            $hasNvidia = $true
            Log-Message "OK NVIDIA GPU detected"
            # Check CUDA
            if (Get-Command nvcc -ErrorAction SilentlyContinue) {
                $hasCuda = $true
                Log-Message "OK CUDA toolkit found"
            } else {
                Log-Message "! CUDA not found - download from https://developer.nvidia.com/cuda-downloads"
            }
        }
    } catch {
        Log-Message "No NVIDIA GPU detected"
    }

    # Check AMD
    $hasAmd = $false
    $hasHip = $false

    # Step 4: Install Python dependencies
    Update-Progress 20 "Installing Python dependencies..."
    Run-Command "pip install --upgrade pip" "Upgrading pip"
    Run-Command "pip install huggingface_hub transformers torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121" "Installing PyTorch and transformers"
    Run-Command "pip install accelerate safetensors" "Installing additional ML libraries"

    # Step 5: Build Rust application
    Update-Progress 30 "Building Rust application..."
    if (Run-Command "cargo build --release" "Building Amni-Code") {
        Log-Message "OK Rust application built successfully"
    } else {
        Log-Message "ERROR Build failed!"
        Update-Progress 0 "Build failed - check logs"
        return
    }

    # Step 6: Create models directory
    Update-Progress 40 "Setting up models directory..."
    if (!(Test-Path "models")) {
        New-Item -ItemType Directory -Path "models" | Out-Null
    }
    Log-Message "OK Models directory created"

    # Step 7: Download models
    Update-Progress 50 "Downloading AI models..."
    Log-Message "Downloading Qwen3.5-9B-Neo (this may take several minutes)..."

    if (Run-Command "huggingface-cli download Jackrong/Qwen3.5-9B-Neo --local-dir models/Qwen3.5-9B-Neo --local-dir-use-symlinks False" "Downloading Qwen3.5-9B-Neo") {
        Log-Message "OK Qwen3.5-9B-Neo downloaded"
    } else {
        Log-Message "! Failed to download Qwen3.5-9B-Neo - you can retry later"
    }

    Update-Progress 75 "Downloading second model..."
    Log-Message "Downloading MLX-Qwen3.5-4B..."

    if (Run-Command "huggingface-cli download Jackrong/MLX-Qwen3.5-4B-Claude-4.6-Opus-Reasoning-Distilled-8bit --local-dir models/MLX-Qwen3.5-4B --local-dir-use-symlinks False" "Downloading MLX-Qwen3.5-4B") {
        Log-Message "OK MLX-Qwen3.5-4B downloaded"
    } else {
        Log-Message "! Failed to download MLX-Qwen3.5-4B - you can retry later"
    }

    # Step 8: Final setup
    Update-Progress 90 "Finalizing installation..."

    # Create desktop shortcut
    try {
        $WshShell = New-Object -comObject WScript.Shell
        $Shortcut = $WshShell.CreateShortcut("$env:USERPROFILE\Desktop\Amni-Code.lnk")
        $Shortcut.TargetPath = "$PWD\target\release\amni-code.exe"
        $Shortcut.WorkingDirectory = "$PWD"
        $Shortcut.Description = "Amni-Code AI Assistant"
        $Shortcut.Save()
        Log-Message "OK Desktop shortcut created"
    } catch {
        Log-Message "! Could not create desktop shortcut"
    }

    Update-Progress 100 "Installation complete!"

    Log-Message ""
    Log-Message "=========================================="
    Log-Message "    INSTALLATION COMPLETE!"
    Log-Message "=========================================="
    Log-Message ""
    Log-Message "Hardware Detected:"
    if ($hasNvidia) {
        Log-Message "  - NVIDIA GPU: YES"
        if ($hasCuda) { Log-Message "  - CUDA: YES" } else { Log-Message "  - CUDA: NO" }
    } else {
        Log-Message "  - NVIDIA GPU: NO"
    }
    if ($hasAmd) {
        Log-Message "  - AMD GPU: YES"
        if ($hasHip) { Log-Message "  - HIP/ROCm: YES" } else { Log-Message "  - HIP/ROCm: NO" }
    } else {
        Log-Message "  - AMD GPU: NO"
    }
    Log-Message ""
    Log-Message "To run Amni-Code:"
    Log-Message "1. Execute: target\release\amni-code.exe"
    Log-Message "2. Open http://localhost:3030 in your browser"
    Log-Message ""
    Log-Message "Desktop shortcut created for easy access."
    Log-Message ""
    if ($hasAmd -and !$hasHip) {
        Log-Message "Note: For AMD users - complete HIP installation and restart."
    }

    [System.Windows.Forms.MessageBox]::Show("Installation complete! Click OK to exit.", "Installation Complete", [System.Windows.Forms.MessageBoxButtons]::OK, [System.Windows.Forms.MessageBoxIcon]::Information)
}

# Install function
$installButton.Add_Click({
    $installButton.Enabled = $false
    $cancelButton.Text = "Close"
    Install-AmniCode
})

    # Step 1: Check Rust
    Update-Progress 5 "Checking Rust installation..."
    if (!(Get-Command rustc -ErrorAction SilentlyContinue)) {
        Log-Message "Rust not found. Installing..."
        Run-Command "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y" "Installing Rust"
        $cargoBin = Join-Path (Join-Path $env:USERPROFILE ".cargo") "bin"
        $env:PATH += ";$cargoBin"
    } else {
        Log-Message "Rust is already installed"
    }

    # Step 2: Check Python
    Update-Progress 10 "Checking Python installation..."
    $pythonFound = $false
    if (Get-Command python -ErrorAction SilentlyContinue) {
        Log-Message "✓ Python found via 'python'"
        $pythonFound = $true
    } elseif (Get-Command py -ErrorAction SilentlyContinue) {
        Log-Message "✓ Python found via 'py'"
        $pythonFound = $true
    }

    if (!$pythonFound) {
        Log-Message "Python not found. Opening Microsoft Store..."
        Start-Process "ms-windows-store://pdp/?productid=9PJPW5LDXLZ5"
        [System.Windows.Forms.MessageBox]::Show("Please install Python 3.8+ from Microsoft Store, then click OK to continue.", "Python Installation Required", [System.Windows.Forms.MessageBoxButtons]::OK, [System.Windows.Forms.MessageBoxIcon]::Information)
    }

    # Step 3: Hardware detection
    Update-Progress 15 "Detecting hardware..."
    $hasNvidia = $false
    $hasAmd = $false
    $hasCuda = $false
    $hasHip = $false

    # Check NVIDIA
    try {
        $nvidiaOutput = nvidia-smi 2>$null
        if ($LASTEXITCODE -eq 0) {
            $hasNvidia = $true
            Log-Message "✓ NVIDIA GPU detected"
            # Check CUDA
            if (Get-Command nvcc -ErrorAction SilentlyContinue) {
                $hasCuda = $true
                Log-Message "✓ CUDA toolkit found"
            } else {
                Log-Message "! CUDA not found - download from https://developer.nvidia.com/cuda-downloads"
            }
        }
    } catch {
        Log-Message "No NVIDIA GPU detected"
    }

    # Check AMD
    $hasAmd = $false
    $hasHip = $false

    # AMD setup prompt
    # if ($hasAmd -and !$hasHip) {
    #     $message = "AMD GPU detected but HIP/ROCm not installed. For best performance, install HIP SDK 7.1.1 from https://www.amd.com/en/developer/rocm.html. Would you like to open the download page now?"
    #     $amdDialog = [System.Windows.Forms.MessageBox]::Show($message, "AMD GPU Setup", [System.Windows.Forms.MessageBoxButtons]::YesNo, [System.Windows.Forms.MessageBoxIcon]::Question)
    #     if ($amdDialog -eq [System.Windows.Forms.DialogResult]::Yes) {
    #         Start-Process "https://www.amd.com/en/developer/rocm.html"
    #     }
    # }

    # Step 4: Install Python dependencies
    Update-Progress 20 "Installing Python dependencies..."
    Run-Command "pip install --upgrade pip" "Upgrading pip"
    Run-Command "pip install huggingface_hub transformers torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121" "Installing PyTorch and transformers"
    Run-Command "pip install accelerate safetensors" "Installing additional ML libraries"

    # Step 5: Build Rust application
    Update-Progress 30 "Building Rust application..."
    if (Run-Command "cargo build --release" "Building Amni-Code") {
        Log-Message "✓ Rust application built successfully"
    } else {
        Log-Message "✗ Build failed!"
        Update-Progress 0 "Build failed - check logs"
        return
    }

    # Step 6: Create models directory
    Update-Progress 40 "Setting up models directory..."
    if (!(Test-Path "models")) {
        New-Item -ItemType Directory -Path "models" | Out-Null
    }
    Log-Message "✓ Models directory created"

    # Step 7: Download models
    Update-Progress 50 "Downloading AI models..."
    Log-Message "Downloading Qwen3.5-9B-Neo (this may take several minutes)..."

    if (Run-Command "huggingface-cli download Jackrong/Qwen3.5-9B-Neo --local-dir models/Qwen3.5-9B-Neo --local-dir-use-symlinks False" "Downloading Qwen3.5-9B-Neo") {
        Log-Message "✓ Qwen3.5-9B-Neo downloaded"
    } else {
        Log-Message "! Failed to download Qwen3.5-9B-Neo - you can retry later"
    }

    Update-Progress 75 "Downloading second model..."
    Log-Message "Downloading MLX-Qwen3.5-4B..."

    if (Run-Command "huggingface-cli download Jackrong/MLX-Qwen3.5-4B-Claude-4.6-Opus-Reasoning-Distilled-8bit --local-dir models/MLX-Qwen3.5-4B --local-dir-use-symlinks False" "Downloading MLX-Qwen3.5-4B") {
        Log-Message "✓ MLX-Qwen3.5-4B downloaded"
    } else {
        Log-Message "! Failed to download MLX-Qwen3.5-4B - you can retry later"
    }

    # Step 8: Final setup
    Update-Progress 90 "Finalizing installation..."

    # Create desktop shortcut
    try {
        $WshShell = New-Object -comObject WScript.Shell
        $Shortcut = $WshShell.CreateShortcut("$env:USERPROFILE\Desktop\Amni-Code.lnk")
        $Shortcut.TargetPath = "$PWD\target\release\amni-code.exe"
        $Shortcut.WorkingDirectory = "$PWD"
        $Shortcut.Description = "Amni-Code AI Assistant"
        $Shortcut.Save()
        Log-Message "✓ Desktop shortcut created"
    } catch {
        Log-Message "! Could not create desktop shortcut"
    }

    Update-Progress 100 "Installation complete!"

    Log-Message ""
    Log-Message "=========================================="
    Log-Message "    INSTALLATION COMPLETE!"
    Log-Message "=========================================="
    Log-Message ""
    Log-Message "Hardware Detected:"
    if ($hasNvidia) {
        Log-Message "  - NVIDIA GPU: YES"
        if ($hasCuda) { Log-Message "  - CUDA: YES" } else { Log-Message "  - CUDA: NO" }
    } else {
        Log-Message "  - NVIDIA GPU: NO"
    }
    if ($hasAmd) {
        Log-Message "  - AMD GPU: YES"
        if ($hasHip) { Log-Message "  - HIP/ROCm: YES" } else { Log-Message "  - HIP/ROCm: NO" }
    } else {
        Log-Message "  - AMD GPU: NO"
    }
    Log-Message ""
    Log-Message "To run Amni-Code:"
    Log-Message "1. Execute: target\release\amni-code.exe"
    Log-Message "2. Open http://localhost:3030 in your browser"
    Log-Message ""
    Log-Message "Desktop shortcut created for easy access."
    Log-Message ""
    if ($hasAmd -and !$hasHip) {
        Log-Message "Note: For AMD users - complete HIP installation and restart."
    }

    [System.Windows.Forms.MessageBox]::Show("Installation complete! Click OK to exit.", "Installation Complete", [System.Windows.Forms.MessageBoxButtons]::OK, [System.Windows.Forms.MessageBoxIcon]::Information)
})

# Cancel button handler
$cancelButton.Add_Click({
    if ($cancelButton.Text -eq "Close") {
        $form.Close()
    } else {
        $form.Close()
    }
})

# Show the form
$form.Add_Shown({$form.Activate()})
[void]$form.ShowDialog()
