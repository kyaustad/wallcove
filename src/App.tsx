import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Button } from "./components/ui/button";
import { env } from "./env";

type DaemonStatus = {
  version: string;
  uptime_secs: number;
  pid: number;
  platform: string;
};

function App() {
  const [daemonMsg, setDaemonMsg] = useState("");
  const [daemonStatus, setDaemonStatus] = useState<DaemonStatus | null>(null);
  const [daemonError, setDaemonError] = useState("");

  async function pingDaemon() {
    setDaemonError("");
    try {
      setDaemonMsg(await invoke<string>("daemon_hello_world"));
      setDaemonStatus(await invoke<DaemonStatus>("daemon_status"));
    } catch (err) {
      setDaemonError(String(err));
      setDaemonMsg("");
      setDaemonStatus(null);
    }
  }

  return (
    <main className="bg-background w-full min-h-screen flex flex-col items-center justify-center gap-4 text-foreground">
      <h1>Wallcove</h1>

      <div className="flex flex-col items-center gap-2">
        <Button onClick={pingDaemon}>Ping Daemon</Button>
        {daemonMsg && <p>{daemonMsg}</p>}
        {daemonStatus && (
          <pre className="text-sm">{JSON.stringify(daemonStatus, null, 2)}</pre>
        )}
        {daemonError && <p className="text-destructive">{daemonError}</p>}
      </div>

      {/* <div className="flex gap-2">
        <Button
          onClick={() =>
            invoke("set_static_image_wallpaper_from_path", {
              path: env.PUBLIC_DEMO_IMAGE_1,
            })
          }
        >
          Bueno
        </Button>
        <Button
          onClick={() =>
            invoke("set_static_image_wallpaper_from_path", {
              path: env.PUBLIC_DEMO_IMAGE_2,
            })
          }
        >
          Blue
        </Button>
      </div> */}

      <div className="flex flex-col items-center gap-2">
        <Button onClick={() => invoke("daemon_shutdown")}>
          Shutdown Daemon
        </Button>
      </div>
    </main>
  );
}

export default App;
