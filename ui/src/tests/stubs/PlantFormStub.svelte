<script lang="ts">
  import type { CreatePlant, Plant } from "$lib/api";

  let {
    initial = null,
    onsave,
  }: {
    initial?: Plant | null;
    onsave?: (data: CreatePlant, photo?: File) => void | Promise<void>;
  } = $props();

  const data: CreatePlant = { name: "Fern" };
  const file = new File(["image"], "fern.jpg", { type: "image/jpeg" });
  let draftName = $state("");

  $effect(() => {
    if (draftName === "") {
      draftName = initial?.name ?? "";
    }
  });
</script>

<div>
  <div data-testid="draft-name">{draftName}</div>
  <button type="button" onclick={() => onsave?.(data)}
    >Save without photo</button
  >
  <button type="button" onclick={() => onsave?.(data, file)}
    >Save with photo</button
  >
</div>
