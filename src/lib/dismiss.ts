// Co che dong chung cho moi popup/menu/dropdown: pointerdown ngoai node (o document) hoac phim Esc thi goi onClose.
// Gan action len phan tu bao ca nut mo lan noi dung popup: bam tren nut mo la "ben trong" nen khong bi dong 2 lan,
// con nut tu no lo toggle. Mo popup khac = pointerdown nam ngoai popup cu nen cai cu tu dong.
export interface DismissOpts {
  open: boolean;
  onClose: () => void;
  /** Selector (trong node) cua nut mo; Esc tra focus ve do de giu dieu huong ban phim. */
  restoreFocus?: string;
}

export function dismiss(node: HTMLElement, opts: DismissOpts) {
  let cur = opts;
  const away = (e: Event) => { if (cur.open && !node.contains(e.target as Node)) cur.onClose(); };
  const esc = (e: KeyboardEvent) => {
    if (!cur.open || e.key !== "Escape") return;
    cur.onClose();
    if (cur.restoreFocus) node.querySelector<HTMLElement>(cur.restoreFocus)?.focus();
  };
  document.addEventListener("pointerdown", away, true);
  document.addEventListener("keydown", esc);
  return {
    update(o: DismissOpts) { cur = o; },
    destroy() { document.removeEventListener("pointerdown", away, true); document.removeEventListener("keydown", esc); },
  };
}
