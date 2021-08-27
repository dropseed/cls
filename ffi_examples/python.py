import os
import json
import platform
from ctypes import cdll

if platform.uname()[0] == "Windows":
    extension = "dll"
if platform.uname()[0] == "Linux":
    extension = "so"
else:
    extension = "dylib"

lib = cdll.LoadLibrary(os.path.join(os.path.dirname(os.path.dirname(__file__)), "target/release/libcls_ffi." + extension))

lib.set_debug(1)
lib.set_project_key("_foo_".encode("utf-8"))
lib.set_project_slug("_slug_".encode("utf-8"))
lib.set_noninteractive_tracking(1, 1)
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
