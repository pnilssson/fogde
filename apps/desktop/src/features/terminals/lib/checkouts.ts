import type { Checkout, Workspace } from "@/generated/protocol";
import { folderName } from "@/lib/folders";

export function checkoutName(checkout: Checkout) {
  return checkout.branch ?? folderName(checkout.path);
}

export function checkoutOf(workspace: Workspace, folder: string) {
  return workspace.checkouts.find(({ path }) => path === folder);
}
