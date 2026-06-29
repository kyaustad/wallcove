import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Button } from "./components/ui/button";

type DaemonStatus = {
  version: string;
  uptime_secs: number;
  pid: number;
  platform: string;
};

type WallpaperKind = "none" | "static_image" | "video";

type ActiveWallpaper = {
  kind: WallpaperKind;
  path?: string;
};

type WallpaperApplied = {
  kind: Exclude<WallpaperKind, "none">;
  path: string;
};

function App() {
  const [daemonMsg, setDaemonMsg] = useState("");
  const [daemonStatus, setDaemonStatus] = useState<DaemonStatus | null>(null);
  const [activeWallpaper, setActiveWallpaper] = useState<ActiveWallpaper | null>(
    null,
  );
  const [lastApplied, setLastApplied] = useState<WallpaperApplied | null>(null);
  const [daemonError, setDaemonError] = useState("");

  async function refreshActiveWallpaper() {
    setActiveWallpaper(await invoke<ActiveWallpaper>("daemon_get_active_wallpaper"));
  }

  async function pingDaemon() {
    setDaemonError("");
    try {
      setDaemonMsg(await invoke<string>("daemon_hello_world"));
      setDaemonStatus(await invoke<DaemonStatus>("daemon_status"));
      await refreshActiveWallpaper();
    } catch (err) {
      setDaemonError(String(err));
      setDaemonMsg("");
      setDaemonStatus(null);
      setActiveWallpaper(null);
    }
  }

  async function applyStaticWallpaper() {
    setDaemonError("");
    try {
      const applied = await invoke<WallpaperApplied>("pick_and_set_static_wallpaper");
      setLastApplied(applied);
      await refreshActiveWallpaper();
    } catch (err) {
      setDaemonError(String(err));
    }
  }

  async function applyVideoWallpaper() {
    setDaemonError("");
    try {
      const applied = await invoke<WallpaperApplied>("pick_and_set_video_wallpaper");
      setLastApplied(applied);
      await refreshActiveWallpaper();
    } catch (err) {
      setDaemonError(String(err));
    }
  }

  async function clearWallpaper() {
    setDaemonError("");
    try {
      setActiveWallpaper(await invoke<ActiveWallpaper>("daemon_clear_wallpaper"));
      setLastApplied(null);
    } catch (err) {
      setDaemonError(String(err));
    }
  }

  return (
    <main className="bg-background w-full min-h-screen flex flex-col items-center justify-center gap-6 text-foreground p-6">
      <div className="text-center space-y-2">
        <h1 className="text-2xl font-semibold">Wallcove</h1>
        <p className="text-sm text-muted-foreground max-w-lg">
          Prototype: Tauri talks to wallcovedaemon over IPC to set static images or
          GPU-decoded video wallpapers.
        </p>
      </div>

      <div className="flex flex-col items-center gap-2">
        <Button onClick={pingDaemon}>Ping Daemon</Button>
        {daemonMsg && <p>{daemonMsg}</p>}
        {daemonStatus && (
          <pre className="text-sm">{JSON.stringify(daemonStatus, null, 2)}</pre>
        )}
        {daemonError && <p className="text-destructive">{daemonError}</p>}
      </div>

      <div className="flex flex-col items-center gap-3 w-full max-w-md">
        <h2 className="text-lg font-medium">Wallpaper</h2>
        <div className="flex flex-wrap gap-2 justify-center">
          <Button onClick={applyStaticWallpaper}>Pick Image Wallpaper</Button>
          <Button onClick={applyVideoWallpaper}>Pick Video Wallpaper</Button>
          <Button variant="outline" onClick={clearWallpaper}>
            Clear Wallpaper
          </Button>
        </div>
        {lastApplied && (
          <p className="text-sm">
            Applied {lastApplied.kind}: {lastApplied.path}
          </p>
        )}
        {activeWallpaper && (
          <pre className="text-sm w-full">
            {JSON.stringify(activeWallpaper, null, 2)}
          </pre>
        )}
      </div>

      <div className="flex flex-col items-center gap-2">
        <Button variant="outline" onClick={() => invoke("daemon_shutdown")}>
          Shutdown Daemon
        </Button>
      </div>
    </main>
  );
}

export default App;
