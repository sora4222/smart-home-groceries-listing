import { type FormEvent, useState } from "react";

import { useGoogleTasksSection } from "#/components/intake/google-tasks-section";
import { Button } from "#/components/ui/button";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "#/components/ui/select";
import { Switch } from "#/components/ui/switch";
import { POLL_CHOICES } from "#/lib/poll-interval";

/**
 * Which list is the grocery inbox, whether to check it, and how often.
 * Checking cannot be switched on until a list is picked.
 */
export function GoogleTasksChoicesForm() {
	const { status, lists, busy, save } = useGoogleTasksSection();
	const [listId, setListId] = useState(status.task_list?.id ?? "");
	const [enabled, setEnabled] = useState(status.enabled);
	const [pollSeconds, setPollSeconds] = useState(status.poll_seconds);

	function submit(event: FormEvent) {
		event.preventDefault();
		void save({
			task_list_id: listId || null,
			enabled: enabled && listId !== "",
			poll_seconds: pollSeconds,
		});
	}

	return (
		<form className="flex flex-col gap-4" onSubmit={submit}>
			<div className="flex flex-col gap-2">
				<label htmlFor="tasks-list" className="text-sm font-medium">
					Grocery list
				</label>
				<Select value={listId} onValueChange={setListId}>
					<SelectTrigger id="tasks-list" className="w-full sm:w-72">
						<SelectValue
							placeholder={lists ? "Pick a list" : "Loading lists…"}
						/>
					</SelectTrigger>
					<SelectContent>
						{(lists ?? (status.task_list ? [status.task_list] : [])).map(
							(list) => (
								<SelectItem key={list.id} value={list.id}>
									{list.title || "(no name)"}
								</SelectItem>
							),
						)}
					</SelectContent>
				</Select>
				<p className="text-xs text-muted-foreground">
					Use a list just for groceries. Every task on it is taken off the list.
				</p>
			</div>

			<div className="flex flex-col gap-2">
				<label htmlFor="tasks-interval" className="text-sm font-medium">
					How often to check
				</label>
				<Select
					value={String(pollSeconds)}
					onValueChange={(value) => setPollSeconds(Number(value))}
				>
					<SelectTrigger id="tasks-interval" className="w-full sm:w-72">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						{POLL_CHOICES.map((choice) => (
							<SelectItem key={choice.seconds} value={String(choice.seconds)}>
								{choice.label}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</div>

			<div className="flex items-center gap-3">
				<Switch
					id="tasks-enabled"
					checked={enabled}
					disabled={listId === ""}
					onCheckedChange={setEnabled}
				/>
				<label htmlFor="tasks-enabled" className="text-sm">
					Check this list
				</label>
			</div>

			<Button
				type="submit"
				className="w-full sm:w-fit"
				disabled={busy !== null}
			>
				Save
			</Button>
		</form>
	);
}
