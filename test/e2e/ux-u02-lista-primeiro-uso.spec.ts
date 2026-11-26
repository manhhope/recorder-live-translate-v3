// U-02 lista de 3 passos no primeiro uso (chave Soniox, permissoes, fonte de audio) com um botao primario.
import { test, expect } from "./fixtures.ts";

for (const loc of ["en", "pt-PT"] as const) {
  test(`U-02 sem chave: lista mostra passo da chave pendente e botao primario desativado ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc, hasKey: false });
    const cl = page.getByTestId("checklist");
    await expect(cl).toBeVisible();
    await expect(page.getByTestId("cl-key")).not.toHaveClass(/done/);
    await expect(page.getByTestId("cl-perm")).toHaveClass(/done/);
    await expect(page.getByTestId("cl-src")).toHaveClass(/done/);
    await expect(page.getByTestId("cl-go")).toBeDisabled();
    await page.getByTestId("cl-key").getByRole("button", { name: rlt.t("cl.key.add") }).click();
    await expect(page.getByRole("dialog", { name: rlt.t("rec.key.title") })).toBeVisible();
    // o texto da chave em falta ja nao fica cortado no rodape: aparece inteiro na lista
    await expect(page.locator(".foot .fs")).toHaveCount(0);
    const w = await page.getByTestId("cl-key").locator("span").evaluate(e => ({ s: e.scrollWidth, c: e.clientWidth }));
    expect(w.s).toBeLessThanOrEqual(w.c + 1);
  });

  test(`U-02 tudo pronto: botao primario ativo e inicia a gravacao como antes ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc });
    await expect(page.getByTestId("cl-go")).toBeEnabled();
    await page.getByTestId("cl-go").click();
    await expect(rlt.btn("tool.stop")).toBeVisible();
  });

  test(`U-02 permissao em falta: passo pendente com botao para rever ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc, perms: { mic: "denied" }, permWelcomeOff: true });
    await expect(page.getByTestId("cl-perm")).not.toHaveClass(/done/);
    await expect(page.getByTestId("cl-go")).toBeDisabled();
    await page.getByTestId("cl-perm").getByRole("button", { name: rlt.t("pw.review") }).click();
    await expect(page.getByRole("dialog")).toBeVisible();
  });
}
