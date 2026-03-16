import os
import sys
import subprocess
import shutil

def main():
    args = sys.argv[1:]
    
    command = shutil.which("async-shell-bin")
    
    if not command:
        # Fallback for local development inside the monorepo
        # Assuming execution from site-packages, we go up to the workspace root
        # Or if executed directly from src, we just need to get to the workspace root
        
        # Determine if we're in site-packages or local src
        current_dir = os.path.dirname(os.path.abspath(__file__))
        
        if "site-packages" in current_dir:
            # Not natively supported by PyPI packaging without careful setup, so rely on the rust-bin being placed nearby
            possible_bin = os.path.join(current_dir, "async_shell")
            if os.path.exists(possible_bin):
                command = possible_bin
        else:
            # We are in local dev (`async-shell/src/async_shell/cli.py`)
            local_binary = os.path.abspath(os.path.join(current_dir, "../../../target/release/async-shell"))
            if os.path.exists(local_binary):
                command = local_binary

    if not command:
        print("\033[91mError: Native `async-shell` binary not found.\033[0m", file=sys.stderr)
        sys.exit(1)

    try:
        result = subprocess.run([command] + args)
        sys.exit(result.returncode)
    except KeyboardInterrupt:
        sys.exit(130)

if __name__ == "__main__":
    main()
