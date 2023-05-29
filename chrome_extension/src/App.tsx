import { MantineProvider, Switch } from "@mantine/core";
import { useEffect, useState } from "react";
import { parse } from "../pkg/backlog_realtime_preview";
import "./App.css";

type Props = {
  editor: HTMLTextAreaElement;
  preview: HTMLElement;
};

const App = ({ editor, preview }: Props) => {
  const [isActive, setIsActive] = useState(true);

  const handleInput = (e: Event) => {
    preview.innerHTML = parse(
      (e.currentTarget as HTMLInputElement | null)?.value ?? ""
    );
  };

  useEffect(() => {
    if (isActive) {
      editor.style.width = "50%";
      preview.style.display = "block";
      preview.innerHTML = parse(editor.value);
      editor.addEventListener("input", handleInput);
    } else {
      editor.style.width = "100%";
      preview.style.display = "none";
      preview.innerHTML = "";
      editor.removeEventListener("input", handleInput);
    }

    return () => {
      editor.removeEventListener("input", handleInput);
    };
  }, [isActive]);

  return (
    <MantineProvider>
      <Switch
        className="switch"
        label="Realtime Preview"
        size="lg"
        checked={isActive}
        onChange={(event) => setIsActive(event.currentTarget.checked)}
      />
    </MantineProvider>
  );
};

export default App;
