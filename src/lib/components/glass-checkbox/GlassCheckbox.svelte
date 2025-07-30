<script lang="ts">
  type Props = {
    label: string;
    checked: boolean;
    disabled?: boolean;
    loading?: boolean;
    onChange: (checked: boolean) => void;
  };
  let {
    label,
    checked,
    onChange,
    disabled: pDisabled = false,
    loading = false,
  }: Props = $props();
  let uid = $props.id();
  let disabled = $derived(loading || pDisabled);
</script>

<button
  id={`${uid}-glass-checkbox`}
  aria-label={label}
  class={["glass-checkbox", { disabled, loading, checked }]}
  {disabled}
  onclick={() => onChange(!checked)}
>
  <input
    {checked}
    aria-label={label}
    name={`${uid}-checkbox`}
    {disabled}
    class="sr-only"
  />
</button>

<style lang="postcss">
  @reference "../../../app";
  .glass-checkbox {
    display: block;
    position: relative;
    padding-left: 35px;
    cursor: pointer;
    user-select: none;
  }

  .glass-checkbox:hover:before {
    @apply border border-primary-900;
    box-shadow: 3px 1px 0 var(--color-primary-400);
  }

  .glass-checkbox.loading:before {
    animation: spin 2s linear infinite;
  }

  .glass-checkbox:before {
    @apply border border-primary-900;
    content: "";
    display: block;
    width: 1.4em;
    height: 1.4em;
    border-radius: 1em;
    position: absolute;
    left: 0;
    top: 0;
    transition:
      all 0.2s,
      background 0.2s ease-in-out;
    background: var(--color-white);
    box-shadow: -3px -1px 0 var(--color-primary-400);
  }
  .glass-checkbox.checked:before {
    @apply rounded-full border-2 border-primary-100 bg-primary-800;
    width: 1.3em;
    height: 1.3em;
    box-shadow: 3px 1px 0 var(--color-primary-400);
  }
</style>
