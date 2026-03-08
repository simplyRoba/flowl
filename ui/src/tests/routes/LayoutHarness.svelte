<script lang="ts">
  import Layout from "../../routes/+layout.svelte";

  interface Props {
    overlay?: "none" | "dialog" | "lightbox" | "chat" | "care-entry";
  }

  let { overlay = "none" }: Props = $props();
</script>

<Layout>
  <div data-testid="layout-page-content">Layout content</div>

  {#if overlay === "dialog"}
    <dialog open>Blocking dialog</dialog>
  {:else if overlay === "lightbox"}
    <dialog class="lightbox" open>Lightbox</dialog>
  {:else if overlay === "chat"}
    <div class="chat-drawer">Chat drawer</div>
  {:else if overlay === "care-entry"}
    <div class="care-entry-form">Inline care form</div>
  {/if}
</Layout>
