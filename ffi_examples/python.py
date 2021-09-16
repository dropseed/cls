import os
import json
import platform
from ctypes import cdll


print(platform.uname())

if platform.uname()[0] == "Windows":
    filename = "cls_ffi.dll"
elif platform.uname()[0] == "Linux":
    filename = "libcls_ffi.so"
else:
    filename = "libcls_ffi.dylib"

# To clear settings
# rm -r /Users/davegaeddert/Library/Application\ Support/_slug__cls__instanceid_/

lib = cdll.LoadLibrary(
    os.path.join(
        os.path.dirname(os.path.dirname(__file__)),
        "target", "release", filename
    )
)

lib.set_debug(1)
lib.set_version("1.0.0".encode("utf-8"))
lib.set_project_key("_foo_".encode("utf-8"))
lib.set_project_slug("_slug_".encode("utf-8"))
lib.set_instance_id("_instanceid_".encode("utf-8"))
lib.set_noninteractive_tracking_enabled(1)  # deprecated
lib.set_ci_tracking_enabled(1)
lib.track_event(
    "_slug_".encode("utf-8"),
    "command".encode("utf-8"),
    json.dumps({"version": "1.0"}).encode("utf-8"),
    0,
)
lib.dispatch_events()
lib.set_request_permission_prompt("_prompt_".encode("utf-8"))
lib.set_user_id("_user_id_".encode("utf-8"))
lib.set_invocation_id("_invocation_id_".encode("utf-8"))
lib.set_is_noninteractive(
    0
)  # deprecated
lib.set_is_ci(
    0
)  # do this last so we use defaults when tracking event but make sure call works
