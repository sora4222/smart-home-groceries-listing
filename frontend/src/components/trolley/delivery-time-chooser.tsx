import { Button } from "#/components/ui/button";
import type { DeliveryTimeOfDay, DeliveryWanted } from "#/lib/api";
import { deliveryDayChoices } from "#/lib/delivery-days";

/** The parts of the day offered, with the hours each one means. */
const TIMES_OF_DAY: {
	value: DeliveryTimeOfDay;
	label: string;
	hint: string;
}[] = [
	{ value: "any", label: "Any time", hint: "cheapest, then earliest" },
	{ value: "morning", label: "Morning", hint: "starts before 12pm" },
	{ value: "afternoon", label: "Afternoon", hint: "starts 12pm–5pm" },
	{ value: "evening", label: "Evening", hint: "starts 5pm or later" },
];

/**
 * Picks the delivery day and part of the day to reserve on the store's
 * website. Tomorrow and "any time" are the defaults. The household can change
 * the reserved time on the store's website later.
 */
export function DeliveryTimeChooser({
	value,
	onChange,
	today = new Date(),
}: {
	value: DeliveryWanted;
	onChange: (value: DeliveryWanted) => void;
	today?: Date;
}) {
	return (
		<div className="flex flex-col gap-3">
			<fieldset className="flex flex-col gap-1">
				<legend className="text-sm font-medium">Delivery day</legend>
				<div className="flex flex-wrap gap-2">
					{deliveryDayChoices(today).map((day) => (
						<Button
							key={day.label}
							type="button"
							size="sm"
							variant={value.date === day.date ? "default" : "outline"}
							aria-pressed={value.date === day.date}
							onClick={() => onChange({ ...value, date: day.date })}
						>
							{day.label}
						</Button>
					))}
				</div>
			</fieldset>
			<fieldset className="flex flex-col gap-1">
				<legend className="text-sm font-medium">Time of day</legend>
				<div className="flex flex-wrap gap-2">
					{TIMES_OF_DAY.map((time) => (
						<Button
							key={time.value}
							type="button"
							size="sm"
							variant={value.time_of_day === time.value ? "default" : "outline"}
							aria-pressed={value.time_of_day === time.value}
							title={time.hint}
							onClick={() => onChange({ ...value, time_of_day: time.value })}
						>
							{time.label}
						</Button>
					))}
				</div>
			</fieldset>
		</div>
	);
}
