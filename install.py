#!/usr/bin/env python3
"""
Amni-Code Installer
Python-based installer for Amni-Code AI assistant
"""

import os
import sys
import subprocess
import platform
import shutil
from pathlib import Path

class AmniCodeInstaller:
    def __init__(self):
        self.project_root = Path(__file__).parent
        self.has_nvidia = False
        self.has_amd = False
        self.has_cuda = False
        self.has_hip = False

    def run_command(self, cmd, description="", check=True, quiet=False):
        """Run a command and return success status"""
        if not quiet:
            print(f"[INFO] {description}")
        if isinstance(cmd, str):
            cmd = cmd.split()

        # Use Python 3.13 specifically for pip commands
        if len(cmd) >= 2 and cmd[0] == "python" and cmd[1] == "-m" and cmd[2] == "pip":
            cmd = ["py", "-3.13", "-m", "pip"] + cmd[3:]
        elif len(cmd) >= 1 and cmd[0] == "python":
            cmd = ["py", "-3.13"] + cmd[1:]

        try:
            result = subprocess.run(cmd, capture_output=True, text=True, cwd=self.project_root)
            success = result.returncode == 0
            if check and not success:
                print(f"[ERROR] Command failed: {' '.join(cmd)}")
                print(f"[ERROR] {result.stderr}")
                return False
            return success
        except Exception as e:
            if check:
                print(f"[ERROR] Failed to run command: {e}")
            return False

    def check_rust(self):
        """Check for Rust and install if needed"""
        print("\n[1/8] Checking for Rust toolchain...")
        if self.run_command("rustc --version", check=False):
            print("Rust is already installed.")
            return True

        print("Rust not found. Installing...")
        rustup_cmd = 'curl --proto \'=https\' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
        if self.run_command(rustup_cmd, "Installing Rust"):
            # Add Rust to PATH for current session
            cargo_bin = Path.home() / ".cargo" / "bin"
            os.environ["PATH"] += os.pathsep + str(cargo_bin)
            print("Rust installed successfully.")
            return True
        return False

    def check_python(self):
        """Check for Python"""
        print("\n[2/8] Checking for Python...")
        
        # Check if Python 3.13 is available
        try:
            result = subprocess.run(["py", "-3.13", "--version"], capture_output=True, text=True)
            if result.returncode == 0:
                version = result.stdout.strip()
                print(f"Python 3.13 detected: {version}")
                print("Using Python 3.13 for installation.")
                return True
            else:
                print("Python 3.13 not found.")
                print("Please install Python 3.13 from https://python.org")
                return False
        except Exception as e:
            print(f"Could not check for Python 3.13: {e}")
            print("Falling back to system Python...")
            # Fallback to system python check
            try:
                import sys
                version = sys.version_info
                python_version = f"{version.major}.{version.minor}.{version.micro}"
                print(f"System Python {python_version} detected.")
                
                if version.major != 3 or version.minor < 8 or version.minor > 13:
                    print(f"WARNING: Python {python_version} may not be compatible.")
                    print("Recommended: Python 3.8-3.13")
                    return False
                return True
            except Exception as e:
                print(f"Could not determine Python version: {e}")
                return False

    def detect_hardware(self):
        """Detect GPU hardware"""
        print("\n[3/8] Detecting hardware acceleration...")

        # Check NVIDIA
        if self.run_command("nvidia-smi", check=False, quiet=True):
            self.has_nvidia = True
            print("NVIDIA GPU detected.")
            if self.run_command("nvcc --version", check=False, quiet=True):
                self.has_cuda = True
                print("CUDA toolkit found.")
            else:
                print("CUDA not found. Download from: https://developer.nvidia.com/cuda-downloads")
        else:
            print("No NVIDIA GPU detected.")

        # Check AMD
        rocm_paths = [Path("C:/Program Files/AMD/ROCm"), Path("/opt/rocm")]
        for path in rocm_paths:
            if path.exists():
                self.has_amd = True
                self.has_hip = True
                print(f"AMD GPU with HIP/ROCm detected ({'WSL' if '/opt' in str(path) else 'Windows'}).")
                break
        else:
            # Try WMIC detection
            try:
                result = subprocess.run(["wmic", "path", "win32_VideoController", "get", "name"],
                                      capture_output=True, text=True)
                if "AMD" in result.stdout.upper() or "RADEON" in result.stdout.upper():
                    self.has_amd = True
                    print("AMD GPU detected but HIP/ROCm not installed.")
                    self.show_amd_setup()
            except:
                pass

        if not self.has_nvidia and not self.has_amd:
            print("No GPU detected. Running in CPU mode.")

        return True

    def show_amd_setup(self):
        """Show AMD GPU setup instructions"""
        print("\n" + "="*50)
        print("    AMD GPU Setup Instructions")
        print("="*50)
        print("\nFor AMD GPUs, you have two options:")
        print("\nOption 1 - Install HIP/ROCm (Recommended):")
        print("- Download HIP SDK 7.1.1 from:")
        print("  https://www.amd.com/en/developer/rocm.html")
        print("- HIP 7.1.1 can run most 7000-series models")
        print("- Rename GFX arch to GFX1100 for compatibility")
        print("\nOption 2 - Use ZLUDA (Alternative):")
        print("- ZLUDA allows CUDA apps to run on AMD GPUs")
        print("- Download from: https://github.com/vosen/ZLUDA")
        print("- Less compatible but easier setup")

        choice = input("\nInstall HIP now? (y/n): ").lower().strip()
        if choice == 'y':
            print("Opening AMD ROCm download page...")
            if platform.system() == "Windows":
                os.startfile("https://www.amd.com/en/developer/rocm.html")
            else:
                subprocess.run(["xdg-open", "https://www.amd.com/en/developer/rocm.html"])
            input("Press Enter after installation completes...")

    def install_python_deps(self):
        """Install Python dependencies"""
        print("\n[4/8] Installing Python dependencies...")
        
        # First upgrade pip
        if not self.run_command("py -3.13 -m pip install --upgrade pip", check=False):
            print("Warning: Could not upgrade pip, continuing...")
        
        # Install PyTorch (separate because it has special index)
        print("Installing PyTorch...")
        pytorch_cmd = "py -3.13 -m pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121"
        pytorch_success = self.run_command(pytorch_cmd, check=False)
        
        # Install core ML packages
        print("Installing core ML packages...")
        ml_cmd = "py -3.13 -m pip install huggingface_hub transformers accelerate safetensors"
        ml_success = self.run_command(ml_cmd, check=False)
        
        # Install huggingface-cli for model downloads
        print("Installing HuggingFace CLI...")
        hf_cmd = "py -3.13 -m pip install huggingface_hub[cli]"
        hf_success = self.run_command(hf_cmd, check=False)
        
        if not (pytorch_success and ml_success):
            print("\nSome packages failed to install.")
            print("This is likely due to Python version compatibility.")
            print("Python 3.14+ is very new and many packages don't support it yet.")
            print("\nRecommended solutions:")
            print("1. Install Python 3.8-3.13 instead")
            print("2. Or use conda/miniconda for better package management")
            print("3. Check package documentation for 3.14 support")
            return False
        
        print("Python dependencies installed successfully.")
        return True

    def build_rust_app(self):
        """Build the Rust application"""
        print("\n[5/8] Building Rust application...")
        if self.run_command("cargo build --release", "Building Amni-Code"):
            print("Rust application built successfully.")
            return True
        else:
            print("Build failed! Please check Rust installation.")
            return False

    def setup_models_dir(self):
        """Create models directory"""
        print("\n[6/8] Setting up models directory...")
        models_dir = self.project_root / "models"
        models_dir.mkdir(exist_ok=True)
        print("Models directory created.")
        return True

    def download_models(self):
        """Download AI models"""
        print("\n[7/8] Downloading AI models...")
        print("This may take several minutes depending on your internet speed...")

        models = [
            ("Jackrong/Qwen3.5-9B-Neo", "models/Qwen3.5-9B-Neo"),
            ("Jackrong/MLX-Qwen3.5-4B-Claude-4.6-Opus-Reasoning-Distilled-8bit", "models/MLX-Qwen3.5-4B")
        ]

        for model_repo, local_dir in models:
            print(f"\nDownloading {model_repo}...")
            cmd = f"huggingface-cli download {model_repo} --local-dir {local_dir} --local-dir-use-symlinks False"
            if not self.run_command(cmd, f"Downloading {model_repo}", check=False):
                print(f"Failed to download {model_repo}. You can retry later or download manually.")

        print("\nModel downloads completed.")
        return True
        return True

    def create_shortcut(self):
        """Create desktop shortcut"""
        print("\n[8/8] Creating desktop shortcut...")
        try:
            if platform.system() == "Windows":
                try:
                    import winshell
                    from win32com.client import Dispatch
                except ImportError:
                    print("win32com not available, skipping shortcut creation.")
                    print("To create a shortcut manually, create a shortcut to: target\\release\\amni-code.exe")
                    return

                desktop = winshell.desktop()
                exe_path = self.project_root / "target" / "release" / "amni-code.exe"
                shortcut_path = os.path.join(desktop, "Amni-Code.lnk")

                shell = Dispatch('WScript.Shell')
                shortcut = shell.CreateShortCut(shortcut_path)
                shortcut.Targetpath = str(exe_path)
                shortcut.WorkingDirectory = str(self.project_root)
                shortcut.Description = "Amni-Code AI Assistant"
                shortcut.save()

                print("Desktop shortcut created.")
            else:
                print("Desktop shortcut creation not implemented for this platform.")
        except Exception as e:
            print(f"Could not create desktop shortcut: {e}")
            print("You can create a shortcut manually to: target/release/amni-code.exe")

    def show_summary(self):
        """Show installation summary"""
        print("\n" + "="*50)
        print("    INSTALLATION COMPLETE!")
        print("="*50)
        print("\nHardware Detected:")
        if self.has_nvidia:
            print("- NVIDIA GPU: YES")
            print(f"- CUDA: {'YES' if self.has_cuda else 'NO'}")
        else:
            print("- NVIDIA GPU: NO")

        if self.has_amd:
            print("- AMD GPU: YES")
            print(f"- HIP/ROCm: {'YES' if self.has_hip else 'NO'}")
        else:
            print("- AMD GPU: NO")

        print("\nTo run Amni-Code:")
        print("1. Execute: target/release/amni-code.exe")
        print("2. Open http://localhost:3000 in your browser")

        if self.has_amd and not self.has_hip:
            print("\nFor AMD users: Complete HIP installation and restart before running.")

    def run(self):
        """Main installation process"""
        print("="*50)
        print("    Amni-Code Agent Installer v0.3.0")
        print("="*50)

        steps = [
            self.check_rust,
            self.check_python,
            self.detect_hardware,
            self.install_python_deps,
            self.build_rust_app,
            self.setup_models_dir,
            self.download_models,
            self.create_shortcut
        ]

        for step in steps:
            if not step():
                print("\nInstallation failed!")
                return False

        self.show_summary()
        return True

if __name__ == "__main__":
    installer = AmniCodeInstaller()
    success = installer.run()
    sys.exit(0 if success else 1)