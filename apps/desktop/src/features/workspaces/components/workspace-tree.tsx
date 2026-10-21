import { SidebarMenuSub } from "@/components/ui/sidebar";
import {
  type RowsProps,
  ViewRows,
} from "@/features/workspaces/components/view-rows";
import { useShown } from "@/features/workspaces/hooks/use-shown";
import { WorktreeItem } from "@/features/workspaces/components/worktree-item";
import { type View, worktrees } from "@/features/workspaces/lib/selection";

interface Props extends RowsProps {
  view: View | undefined;
  down: boolean;
}

export function WorkspaceTree({ view, down, ...props }: Props) {
  const { machine, workspace } = props;
  const own = { machine: machine.id, id: workspace.id };
  const shown = useShown(workspace, own);
  const linked = worktrees(workspace);
  if (!shown && linked.length === 0) {
    return null;
  }

  return (
    <SidebarMenuSub>
      <ViewRows {...props} view={own} />
      {linked.map((checkout) => (
        <WorktreeItem
          key={checkout.path}
          {...props}
          checkout={checkout}
          current={view}
          down={down}
        />
      ))}
    </SidebarMenuSub>
  );
}
