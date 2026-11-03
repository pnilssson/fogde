import { fileLimit } from "@/generated/protocol";
import { bracket } from "@/lib/paste";

interface Paste {
  text: (data: string) => void;
  file: (name: string, data: string) => Promise<void>;
  failed: (error: unknown) => void;
}

export function handleClipboard(element: HTMLElement, paste: Paste) {
  let queue = Promise.resolve();

  const attach = (files: File[]) => {
    for (const file of files) {
      if (file.size > fileLimit) {
        paste.failed(
          new Error(
            `${file.name} is larger than ${String(fileLimit / 1024 / 1024)} MB.`,
          ),
        );
        continue;
      }

      queue = queue
        .then(() => encode(file))
        .then((data) => paste.file(file.name, data))
        .catch(paste.failed);
    }
  };

  const pasted = (event: ClipboardEvent) => {
    event.preventDefault();
    event.stopPropagation();
    const data = event.clipboardData;
    if (!data) {
      return;
    }

    const text = data.getData("text/plain");
    if (text) {
      paste.text(bracket(text));
      return;
    }

    attach([...data.files]);
  };
  const over = (event: DragEvent) => {
    if (event.dataTransfer?.types.includes("Files")) {
      event.preventDefault();
      event.dataTransfer.dropEffect = "copy";
    }
  };
  const dropped = (event: DragEvent) => {
    if (!event.dataTransfer?.types.includes("Files")) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();
    attach([...event.dataTransfer.files]);
  };

  element.addEventListener("paste", pasted, { capture: true });
  element.addEventListener("dragover", over);
  element.addEventListener("drop", dropped);

  return () => {
    element.removeEventListener("paste", pasted, { capture: true });
    element.removeEventListener("dragover", over);
    element.removeEventListener("drop", dropped);
  };
}

function encode(file: File) {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const url = typeof reader.result === "string" ? reader.result : "";
      resolve(url.slice(url.indexOf(",") + 1));
    };
    reader.onerror = () => {
      reject(
        new Error(`Could not read ${file.name}. Drop files, not folders.`),
      );
    };
    reader.readAsDataURL(file);
  });
}
