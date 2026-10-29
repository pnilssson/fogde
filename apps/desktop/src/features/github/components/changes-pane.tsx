import { type ReactNode, useMemo, useState } from "react";

import { Layer, SideScroll } from "@/components/frame";
import { ChangeList } from "@/features/github/components/change-list";
import { CommitBox } from "@/features/github/components/commit-box";
import type { CheckoutStatus } from "@/generated/protocol";
import type { Machine } from "@/lib/machines";

interface Props {
  machine: Machine;
  checkout: CheckoutStatus;
  header: ReactNode;
}

export function ChangesPane({ machine, checkout, header }: Props) {
  const files = useMemo(
    () => [...checkout.files].sort((a, b) => a.path.localeCompare(b.path)),
    [checkout.files],
  );
  const [skipped, setSkipped] = useState<ReadonlySet<string>>(new Set());
  const chosen = files.filter((file) => !skipped.has(file.path));
  const picked = new Set(chosen.map((file) => file.path));

  return (
    <>
      <Layer className="flex-1">
        {header}
        <SideScroll>
          {files.length === 0 ? (
            <p className="px-2 type-body text-muted-foreground">
              No changes to commit.
            </p>
          ) : (
            <ChangeList
              files={files}
              picked={picked}
              truncated={checkout.truncated}
              onPick={(paths, on) => {
                setSkipped((current) => {
                  const next = new Set(current);
                  for (const path of paths) {
                    if (on) {
                      next.delete(path);
                    } else {
                      next.add(path);
                    }
                  }

                  return next;
                });
              }}
            />
          )}
        </SideScroll>
      </Layer>
      {(files.length > 0 || checkout.last?.undoable) && (
        <Layer aria-label="Commit" className="shrink-0 p-2">
          <CommitBox machine={machine} checkout={checkout} chosen={chosen} />
        </Layer>
      )}
    </>
  );
}
