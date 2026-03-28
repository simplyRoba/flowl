<script lang="ts">
  import { translations } from "$lib/stores/locale";
  import Logo from "./Logo.svelte";
</script>

<div class="offline-message">
  <div class="offline-logo">
    <Logo size={48} />
  </div>
  <p>{$translations.common.offlineMessage}</p>
</div>

<style>
  .offline-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 32px 16px;
    gap: 12px;
  }

  .offline-logo {
    color: var(--color-primary);
  }

  .offline-message p {
    color: var(--color-text-muted);
    font-size: 15px;
    max-width: 320px;
    line-height: 1.5;
    margin: 0;
  }
</style>
