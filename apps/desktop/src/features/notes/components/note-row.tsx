import { FileTextIcon } from "@phosphor-icons/react";
import type { ComponentProps } from "react";

import {
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar";
import {
  type EntryActions,
  EntryMenu,
} from "@/features/notes/components/entry-menu";
import type { Note } from "@/generated/protocol";
import { noteDragProps } from "@/lib/note-drag";

export interface RowActions extends EntryActions {
  onOpen: (path: string) => void;
}

interface Props extends RowActions {
  machine: string;
  note: Note;
  active: boolean;
  nested: boolean;
}

export function NoteRow({
  machine,
  note,
  active,
  nested,
  onOpen,
  onRename,
  onDelete,
}: Props) {
  const Item = nested ? SidebarMenuSubItem : SidebarMenuItem;
  const Button = nested ? SubButton : SidebarMenuButton;

  return (
    <Item>
      <EntryMenu
        kind="note"
        name={note.name}
        path={note.path}
        trigger={
          <Button
            isActive={active}
            {...noteDragProps({ machine, path: note.path })}
            onClick={() => {
              onOpen(note.path);
            }}
          />
        }
        onRename={onRename}
        onDelete={onDelete}
      >
        <FileTextIcon />
        <span className="min-w-0 flex-1 truncate">{note.name}</span>
      </EntryMenu>
    </Item>
  );
}

export function SubButton(props: ComponentProps<typeof SidebarMenuSubButton>) {
  return <SidebarMenuSubButton render={<button type="button" />} {...props} />;
}
