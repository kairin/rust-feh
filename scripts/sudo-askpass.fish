# GUI password prompt for sudo when a display is available (agents + desktop).
if test -x "$HOME/.local/bin/sudo-askpass"
    set -gx SUDO_ASKPASS "$HOME/.local/bin/sudo-askpass"
end

function sudo --description "sudo with GUI askpass when DISPLAY/Wayland is set"
    if set -q SUDO_ASKPASS; and test -x $SUDO_ASKPASS
        if set -q DISPLAY; or set -q WAYLAND_DISPLAY
            command sudo -A $argv
            return $status
        end
    end
    command sudo $argv
end