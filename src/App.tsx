import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Button } from "./components/ui/button";
import { env } from "./env";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main className="bg-background w-full min-h-screen flex flex-col items-center justify-center text-foreground">
      <h1>Hello World</h1>
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
    </main>
  );
}

export default App;
