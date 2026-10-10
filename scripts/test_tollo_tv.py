"""Run with python3 -m unittest discover -s scripts -p 'test_*.py'."""

import contextlib
import io
import json
from pathlib import Path
import runpy
import shutil
import subprocess
import tempfile
import time
import unittest
from unittest.mock import patch
import wave


SCRIPT = Path(__file__).with_name("tollo-tv")
HELPER = runpy.run_path(str(SCRIPT))


class TransportTests(unittest.TestCase):
    def test_url_is_data_not_remote_shell_source(self):
        url = 'https://example.test/live?a="x"&token=$(touch /tmp/nope);`id`'
        result = subprocess.CompletedProcess([], 0, '{"error":"success"}', "")
        with patch("subprocess.run", return_value=result) as run:
            HELPER["send"]("hp", ["loadfile", url, "replace"])
        args, kwargs = run.call_args
        self.assertNotIn(url, args[0][-1])
        self.assertEqual(json.loads(kwargs["input"])["command"][1], url)
        self.assertFalse(kwargs.get("shell", False))

    def test_ssh_failure_is_reported(self):
        result = subprocess.CompletedProcess([], 255, "", "Permission denied")
        with patch("subprocess.run", return_value=result):
            with self.assertRaisesRegex(RuntimeError, "Permission denied"):
                HELPER["send"]("hp", ["stop"])


@unittest.skipUnless(shutil.which("mpv"), "mpv is required for IPC integration tests")
class PlayerTests(unittest.TestCase):
    def test_switch_controls_and_player_failure(self):
        with tempfile.TemporaryDirectory(prefix="tollo-tv-test-") as directory:
            root = Path(directory)
            path = root / "mpv.sock"
            player = subprocess.Popen([
                "mpv", "--no-config", "--idle=yes", "--vo=null", "--ao=null",
                "--input-ipc-server=" + str(path),
            ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            try:
                deadline = time.monotonic() + 5
                while not path.exists():
                    if player.poll() is not None or time.monotonic() > deadline:
                        self.fail("Test mpv did not create its IPC socket")
                    time.sleep(0.02)

                def command(value):
                    data = io.TextIOWrapper(io.BytesIO(json.dumps({"command": value}).encode()))
                    output = io.StringIO()
                    with patch.dict(HELPER["receive"].__globals__, {"socket_path": lambda: path}):
                        with patch("sys.stdin", data), contextlib.redirect_stdout(output):
                            HELPER["receive"]()
                    return json.loads(output.getvalue()).get("data")

                self.assertTrue(command(["get_property", "idle-active"]))
                for name in ("first.wav", "second.wav"):
                    media = root / name
                    with wave.open(str(media), "wb") as audio:
                        audio.setparams((1, 2, 8000, 0, "NONE", "not compressed"))
                        audio.writeframes(b"\0\0" * 8000 * 30)
                    command(["loadfile", str(media), "replace"])
                    deadline = time.monotonic() + 5
                    while command(["get_property", "idle-active"]):
                        if time.monotonic() > deadline:
                            self.fail("Test media did not load")
                        time.sleep(0.02)
                    self.assertEqual(command(["get_property", "path"]), str(media))
                    self.assertIsNone(player.poll())
                command(["set_property", "volume", 35])
                self.assertEqual(command(["get_property", "volume"]), 35)
                command(["cycle", "pause"])
                self.assertTrue(command(["get_property", "pause"]))
                command(["stop"])
                with self.assertRaisesRegex(RuntimeError, "mpv:"):
                    command(["not-a-real-command"])
                player.terminate()
                player.wait(timeout=5)
                with self.assertRaisesRegex(RuntimeError, "not running"):
                    command(["stop"])
            finally:
                if player.poll() is None:
                    player.terminate()
                    player.wait(timeout=5)
