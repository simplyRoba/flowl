<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { page } from "$app/state";
  import Logo from "$lib/components/Logo.svelte";
  import { fetchAuthConfig, safeLocalTarget, type AuthConfig } from "$lib/auth";
  import { translations } from "$lib/stores/locale";

  let config: AuthConfig = $state({
    enabled: true,
    provider_name: "OpenID Connect",
  });
  let configUnavailable = $state(false);
  let loaded = $state(false);

  const returnTo = $derived(
    safeLocalTarget(page.url.searchParams.get("return_to")),
  );
  const loginHref = $derived(
    `/auth/login?return_to=${encodeURIComponent(returnTo)}`,
  );
  const status = $derived.by(() => {
    if (
      configUnavailable ||
      page.url.searchParams.get("error") === "provider_unavailable"
    ) {
      return $translations.auth.providerUnavailable;
    }
    if (page.url.searchParams.get("error") === "authentication_failed") {
      return $translations.auth.authenticationFailed;
    }
    if (page.url.searchParams.get("logged_out") === "1")
      return $translations.auth.loggedOut;
    return null;
  });

  $effect(() => {
    void (async () => {
      try {
        config = await fetchAuthConfig();
        if (!config.enabled) await goto(resolve("/"), { replaceState: true });
      } catch {
        configUnavailable = true;
      } finally {
        loaded = true;
      }
    })();
  });
</script>

<svelte:head><title>flowl — {$translations.auth.title}</title></svelte:head>

<div class="login-page">
  <section class="login-card" aria-busy={!loaded}>
    <div class="brand-area">
      <div class="brand"><Logo size={38} /><span>flowl</span></div>
      <h1>{$translations.auth.title}</h1>
      <p>{$translations.auth.required}</p>
    </div>
    <div class="action-area">
      <div class="action-panel">
        {#if status}<p class="status" role="status">{status}</p>{/if}
        <!-- Backend-only auth endpoint; its target is derived from resolve('/login'). -->
        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
        <a class="btn btn-primary provider-action" href={loginHref}>
          {$translations.auth.continueWith.replace(
            "{provider}",
            config.provider_name ?? "OpenID Connect",
          )}
        </a>
      </div>
    </div>
  </section>
</div>

<style>
  .login-page {
    min-height: 100dvh;
    display: grid;
    place-items: center;
    padding: 24px;
    box-sizing: border-box;
  }
  .login-card {
    width: min(100%, 400px);
    overflow: hidden;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-card);
    background: var(--color-surface);
    box-shadow: 0 8px 24px
      color-mix(in srgb, var(--color-text) 10%, transparent);
  }
  .brand-area {
    padding: 32px 28px 20px;
    text-align: center;
    background: var(--color-surface-muted);
  }
  .brand {
    display: inline-flex;
    align-items: center;
    gap: 9px;
    color: var(--color-primary);
    font-size: 22px;
    font-weight: 700;
  }
  h1 {
    margin: 24px 0 10px;
    font-size: 26px;
    line-height: 1.15;
  }
  p {
    margin: 0;
    color: var(--color-text-muted);
    line-height: 1.5;
  }
  .action-area {
    padding: 20px 28px 28px;
  }
  .action-panel {
    width: 100%;
    box-sizing: border-box;
    padding: 20px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-card);
    background: var(--color-surface);
    box-shadow: 0 6px 16px
      color-mix(in srgb, var(--color-text) 14%, transparent);
  }
  .status {
    margin-bottom: 16px;
    font-size: 14px;
    color: var(--color-danger-text);
  }
  .provider-action {
    width: 100%;
    min-height: 44px;
    box-sizing: border-box;
    justify-content: center;
    text-decoration: none;
  }
  @media (min-width: 48rem) {
    .login-card {
      width: min(100%, 880px);
      min-height: 380px;
      display: grid;
      grid-template-columns: minmax(0, 1.15fr) minmax(300px, 0.85fr);
    }
    .brand-area {
      display: flex;
      flex-direction: column;
      justify-content: center;
      padding: 48px;
      text-align: left;
      border-right: 1px solid var(--color-border);
    }
    .brand {
      align-self: flex-start;
    }
    .action-area {
      display: flex;
      align-items: center;
      padding: 40px;
    }
  }
</style>
