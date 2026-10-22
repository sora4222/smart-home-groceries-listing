import { createFileRoute, useRouter } from "@tanstack/react-router";
import { toast } from "sonner";

import { DeliverySettingsForm } from "#/components/delivery/delivery-settings-form";
import { api, type DeliverySettings } from "#/lib/api";

export const Route = createFileRoute("/settings/delivery")({
	loader: () => api.deliverySettings.get(),
	component: DeliveryPage,
});

/**
 * Settings › Delivery — each store's delivery fee rules and how the order
 * screen picks stores (spec: "Delivery constraints", "Optimisation modes").
 *
 * The app cannot read the stores' fees without your store login, so they are
 * typed here once. Saving shows a toast with Undo, which saves what was there
 * before. The loader stays the single source of truth.
 */
function DeliveryPage() {
	const settings = Route.useLoaderData();
	const router = useRouter();

	async function save(next: DeliverySettings, previous: DeliverySettings) {
		try {
			await api.deliverySettings.save(next);
		} catch (error) {
			toast.error("Could not save the delivery settings — try again.");
			throw error;
		}
		await router.invalidate();
		toast.success("Delivery settings saved", {
			action: { label: "Undo", onClick: () => undo(previous) },
		});
	}

	async function undo(previous: DeliverySettings) {
		try {
			await api.deliverySettings.save(previous);
			toast.success("Undone");
		} catch {
			toast.error("Could not undo — check the settings.");
		}
		await router.invalidate();
	}

	return (
		<div className="flex flex-col gap-4">
			<div className="flex flex-col gap-1">
				<h1 className="text-lg font-semibold">Delivery</h1>
				<p className="text-sm text-muted-foreground">
					The app cannot see the stores' delivery fees, so type them here. The
					order screen adds them to every option.
				</p>
			</div>
			<DeliverySettingsForm
				key={JSON.stringify(settings)}
				settings={settings}
				onSave={(next) => save(next, settings)}
			/>
		</div>
	);
}
