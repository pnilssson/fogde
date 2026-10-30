import { useRef, useState } from "react";

interface Props {
  name: string;
  onRename: (name: string) => Promise<unknown>;
  onDone: () => void;
}

export function NoteTitle({ name, onRename, onDone }: Props) {
  const [value, setValue] = useState(name);
  const cancelled = useRef(false);

  const commit = async (text: string) => {
    const next = text.trim();
    if (cancelled.current || next === "" || next === name) {
      cancelled.current = false;
      setValue(name);

      return;
    }

    try {
      await onRename(next);
    } catch {
      setValue(name);
    }
  };

  return (
    <div className="mx-auto w-full max-w-prose pt-8 pb-4 type-note">
      <input
        aria-label="Note name"
        className="w-full truncate bg-transparent type-display outline-none"
        value={value}
        spellCheck={false}
        onChange={(event) => {
          setValue(event.target.value);
        }}
        onBlur={(event) => {
          void commit(event.currentTarget.value);
        }}
        onKeyDown={(event) => {
          event.stopPropagation();
          if (event.key === "Enter") {
            event.preventDefault();
            onDone();
          } else if (event.key === "Escape") {
            event.preventDefault();
            cancelled.current = true;
            onDone();
          }
        }}
      />
    </div>
  );
}
