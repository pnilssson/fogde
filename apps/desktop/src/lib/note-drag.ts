export const noteDragType = "application/x-fogde-note";

export interface NoteDrag {
  machine: string;
  path: string;
}

let dragging: NoteDrag | undefined;

export function noteDragProps(note: NoteDrag) {
  return {
    draggable: true,
    onDragStart: (event: { dataTransfer: DataTransfer }) => {
      event.dataTransfer.effectAllowed = "copy";
      event.dataTransfer.setData(noteDragType, note.path);
      dragging = note;
    },
    onDragEnd: () => {
      dragging = undefined;
    },
  };
}

export function draggingNote() {
  return dragging;
}
