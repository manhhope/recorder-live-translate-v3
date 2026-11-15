// Lenh IPC treo (WebView2 tren Windows, luong UI ban) khong bao gio tra ve nen giao dien cho mai va tab trong tron.
// `withTimeout` bien "treo" thanh loi `ipc_timeout` de tab hien thong bao kem nut thu lai.
export const IPC_TIMEOUT_MS = 8000;

export function withTimeout<T>(p: Promise<T>, ms: number = IPC_TIMEOUT_MS): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const t = setTimeout(() => reject("ipc_timeout"), ms);
    p.then(v => { clearTimeout(t); resolve(v); }, e => { clearTimeout(t); reject(e); });
  });
}
