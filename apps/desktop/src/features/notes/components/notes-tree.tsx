import { CaretRightIcon, FolderSimpleIcon } from "@phosphor-icons/react";
import { cn } from "cn";
import { type ComponentProps, useState } from "react";

import { Collapsible, CollapsibleContent } from "@/components/ui/collapsible";
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar";
import { NoteRow, type RowActions } from "@/features/notes/components/note-row";
import { type Folder, foldersAbove } from "@/features/notes/lib/tree";

interface Props {
  machine: string;
  root: Folder;
  note: string | undefined;
  actions: RowActions;
}

interface RowsProps {
  machine: string;
  folder: Folder;
  nested: boolean;
  note: string | undefined;
  open: Set<string>;
  onToggle: (path: string) => void;
  actions: RowActions;
}

export function NotesTree({ machine, root, note, actions }: Props) {
  const [open, setOpen] = useState(
    () => new Set(note ? foldersAbove(note, root.path) : []),
  );

  const [shown, setShown] = useState(note);
  if (note !== shown) {
    setShown(note);
    if (note) {
      setOpen(new Set([...open, ...foldersAbove(note, root.path)]));
    }
  }

  const toggle = (path: string) => {
    setOpen((current) => {
      const next = new Set(current);
      if (!next.delete(path)) {
        next.add(path);
      }

      return next;
    });
  };

  return (
    <SidebarMenu className="p-2">
      <Rows
        machine={machine}
        folder={root}
        nested={false}
        note={note}
        open={open}
        onToggle={toggle}
        actions={actions}
      />
    </SidebarMenu>
  );
}

function Rows({
  machine,
  folder,
  nested,
  note,
  open,
  onToggle,
  actions,
}: RowsProps) {
  const Item = nested ? SidebarMenuSubItem : SidebarMenuItem;
  const Button = nested ? SubButton : SidebarMenuButton;

  return (
    <>
      {folder.folders.map((inner) => {
        const expanded = open.has(inner.path);

        return (
          <Item key={inner.path}>
            <Collapsible open={expanded}>
              <Button
                aria-expanded={expanded}
                onClick={() => {
                  onToggle(inner.path);
                }}
              >
                <CaretRightIcon
                  className={cn(
                    "transition-transform ease-out motion-reduce:transition-none",
                    expanded && "rotate-90",
                  )}
                />
                <FolderSimpleIcon />
                <span className="min-w-0 flex-1 truncate">{inner.name}</span>
              </Button>
              <CollapsibleContent>
                {(inner.folders.length > 0 || inner.notes.length > 0) && (
                  <SidebarMenuSub>
                    <Rows
                      machine={machine}
                      folder={inner}
                      nested
                      note={note}
                      open={open}
                      onToggle={onToggle}
                      actions={actions}
                    />
                  </SidebarMenuSub>
                )}
              </CollapsibleContent>
            </Collapsible>
          </Item>
        );
      })}
      {folder.notes.map((current) => (
        <NoteRow
          key={current.path}
          machine={machine}
          note={current}
          active={current.path === note}
          nested={nested}
          {...actions}
        />
      ))}
    </>
  );
}

function SubButton(props: ComponentProps<typeof SidebarMenuSubButton>) {
  return <SidebarMenuSubButton render={<button type="button" />} {...props} />;
}
