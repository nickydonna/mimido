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
    margin-right: 5px;
    cursor: pointer;
    user-select: none;
  }

  .glass-checkbox:hover:before {
    @apply border border-primary-900;
  }

  .glass-checkbox.loading:before {
    @apply animate-ping;
  }

  .glass-checkbox {
    @apply border-3 rounded-full border-primary-800 size-6;
  }
  .glass-checkbox.checked {
    @apply border-3 rounded-full border-primary-800 bg-primary-500 size-6;
  }
</style>
