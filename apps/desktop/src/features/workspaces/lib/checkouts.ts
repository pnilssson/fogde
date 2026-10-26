import { busy } from "@/features/workspaces/lib/status";
import { terminalsOf } from "@/features/workspaces/lib/tile-kinds";
import type { Checkout, Workspace } from "@/generated/protocol";
import { folderName } from "@/lib/folders";

export function checkoutName(checkout: Checkout) {
  return checkout.branch ?? folderName(checkout.path);
}

export function checkoutOf(workspace: Workspace, folder: string) {
  return workspace.checkouts.find(({ path }) => path === folder);
}

export function mainCheckout(workspace: Workspace) {
  return workspace.checkouts.find(({ main }) => main);
}

export function working(workspace: Workspace, checkout: string) {
  return terminalsOf(workspace).filter(
    (terminal) => terminal.checkout === checkout && busy(terminal.status),
  );
}
