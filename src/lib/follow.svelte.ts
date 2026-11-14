// Vung chu tu cuon xuong cuoi: dang o cuoi thi bam theo chu moi; nguoi dung cuon len thi dung va hien nut "Jump to latest";
// cuon toi cuoi (hoac bam nut) thi bam lai. Theo doi bang MutationObserver nen khong phu thuoc luong du lieu cua man hinh.
const SLACK = 24;
const reduced = () => typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;

export class Follow {
  /** true khi nguoi dung dang doc lai o phia tren (hien nut Jump to latest). */
  away = $state(false);
  #node: HTMLElement | null = null;
  #stick = true;
  #jumping = false;

  #atEnd = () => !!this.#node && this.#node.scrollHeight - this.#node.scrollTop - this.#node.clientHeight <= SLACK;
  #snap = () => { if (this.#node) this.#node.scrollTop = this.#node.scrollHeight; };

  action = (node: HTMLElement) => {
    this.#node = node;
    const onScroll = () => {
      if (this.#jumping) { if (this.#atEnd()) this.#jumping = false; else return; }
      this.#stick = this.#atEnd();
      this.away = !this.#stick;
    };
    const onNew = () => { if (this.#stick) this.#snap(); };
    node.addEventListener("scroll", onScroll, { passive: true });
    const mo = new MutationObserver(onNew);
    mo.observe(node, { childList: true, subtree: true, characterData: true });
    const ro = typeof ResizeObserver === "function" ? new ResizeObserver(onNew) : null;
    ro?.observe(node);
    this.#snap();
    return { destroy: () => { node.removeEventListener("scroll", onScroll); mo.disconnect(); ro?.disconnect(); this.#node = null; } };
  };

  jump = () => {
    const n = this.#node;
    if (!n) return;
    this.#stick = true;
    this.away = false;
    if (reduced()) { this.#snap(); return; }
    this.#jumping = true;
    n.scrollTo({ top: n.scrollHeight, behavior: "smooth" });
  };
}
