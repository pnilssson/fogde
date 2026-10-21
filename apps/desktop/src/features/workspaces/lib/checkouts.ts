import type { Checkout, Workspace } from "@/generated/protocol";
import { folderName, inside } from "@/lib/folders";

export function checkoutName(checkout: Checkout) {
  return checkout.branch ?? folderName(checkout.path);
}

export function checkoutOf(workspace: Workspace, folder: string) {
  return workspace.checkouts.find(({ path }) => path === folder);
}

export function mainCheckout(workspace: Workspace) {
  return workspace.checkouts.find(({ main }) => main);
}

export function busy(workspace: Workspace, checkout: string) {
  return workspace.terminals.some(
    (terminal) =>
      terminal.checkout === checkout || inside(terminal.folder, checkout),
  );
}
