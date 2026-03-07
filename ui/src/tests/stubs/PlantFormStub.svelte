<script lang="ts">
  import type { CreatePlant } from "$lib/api";

  let {
    onsave,
  }: {
    onsave?: (data: CreatePlant, photo?: File) => void | Promise<void>;
  } = $props();

  const data: CreatePlant = { name: "Fern" };
  const file = new File(["image"], "fern.jpg", { type: "image/jpeg" });
</script>

<div>
  <button type="button" onclick={() => onsave?.(data)}>Save without photo</button>
  <button type="button" onclick={() => onsave?.(data, file)}>Save with photo</button>
</div>
