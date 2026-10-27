import { busy } from "@/features/workspaces/lib/status";
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
  return workspace.terminals.filter(
    (terminal) => terminal.checkout === checkout && busy(terminal.status),
  );
}
