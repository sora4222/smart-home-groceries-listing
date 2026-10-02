import { createContext, type ReactNode, useContext } from "react";

import { GoogleTasksActivity } from "#/components/intake/google-tasks-activity";
import { GoogleTasksChoicesForm } from "#/components/intake/google-tasks-choices";
import { GoogleTasksConnect } from "#/components/intake/google-tasks-connect";
import { Badge } from "#/components/ui/badge";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "#/components/ui/card";
import { useGoogleTasks } from "#/hooks/useGoogleTasks";
import type { GoogleTasksStatus } from "#/lib/api";

type GoogleTasksState = ReturnType<typeof useGoogleTasks>;

const GoogleTasksContext = createContext<GoogleTasksState | null>(null);

/** The section's state and actions, for the parts inside it. */
export function useGoogleTasksSection(): GoogleTasksState {
	const state = useContext(GoogleTasksContext);
	if (!state) throw new Error("must be inside <GoogleTasksSection>");
	return state;
}

/**
 * The Google Tasks card on `/settings/intake`: connect the account, pick the
 * list that is the grocery inbox, switch checking on, and see how the last
 * check went. The parts share state through a context local to this card.
 */
export function GoogleTasksSection({
	initial,
	children,
}: {
	initial: GoogleTasksStatus;
	children?: ReactNode;
}) {
	const state = useGoogleTasks(initial);
	const { status } = state;

	return (
		<GoogleTasksContext.Provider value={state}>
			<Card data-testid="google-tasks">
				<CardHeader className="flex-row flex-wrap items-start justify-between gap-2">
					<div className="flex flex-col gap-1">
						<CardTitle className="text-base">Google Tasks</CardTitle>
						<CardDescription>
							Add items to a Google Tasks list. The app picks them up, then
							removes them from the list.
						</CardDescription>
					</div>
					<Badge variant={status.polling ? "default" : "outline"}>
						{status.polling ? "On" : "Off"}
					</Badge>
				</CardHeader>
				<CardContent className="flex flex-col gap-4">
					{children ?? (
						<>
							<GoogleTasksConnect />
							{status.connected && <GoogleTasksChoicesForm />}
							{status.connected && <GoogleTasksActivity />}
						</>
					)}
				</CardContent>
			</Card>
		</GoogleTasksContext.Provider>
	);
}
