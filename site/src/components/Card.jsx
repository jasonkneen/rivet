import clsx from "clsx";
import Link from "next/link";

function ChevronRightIcon(props) {
	return (
		<svg viewBox="0 0 16 16" fill="none" aria-hidden="true" {...props}>
			<path
				d="M6.75 5.75 9.25 8l-2.5 2.25"
				strokeWidth="1.5"
				strokeLinecap="round"
				strokeLinejoin="round"
			/>
		</svg>
	);
}

export function Card({ as: Component = "div", className, children }) {
	return (
		<Component
			className={clsx(
				className,
				"group relative flex flex-col items-start",
			)}
		>
			{children}
		</Component>
	);
}

Card.Link = function CardLink({ children, ...props }) {
	return (
		<>
			<div className="absolute -inset-x-4 -inset-y-6 z-0 scale-95 bg-cream-50 opacity-0 transition group-hover:scale-100 group-hover:opacity-100 dark:bg-charcole-800/50 sm:-inset-x-6 sm:rounded-2xl" />
			<Link {...props}>
				<span className="absolute -inset-x-4 -inset-y-6 z-20 sm:-inset-x-6 sm:rounded-2xl" />
				<span className="relative z-10">{children}</span>
			</Link>
		</>
	);
};

Card.Title = function CardTitle({ as: Component = "h2", href, children }) {
	return (
		<Component className="text-base font-semibold tracking-tight text-charcole-800 dark:text-cream-100">
			{href ? <Card.Link href={href}>{children}</Card.Link> : children}
		</Component>
	);
};

Card.Description = function CardDescription({ children }) {
	return (
		<p className="relative z-10 mt-2 text-sm text-charcole-600 dark:text-cream-400">
			{children}
		</p>
	);
};

Card.Cta = function CardCta({ children }) {
	return (
		<div
			aria-hidden="true"
			className="relative z-10 mt-4 flex items-center text-sm font-medium text-violet-500"
		>
			{children}
			<ChevronRightIcon className="ml-1 h-4 w-4 stroke-current" />
		</div>
	);
};

Card.Eyebrow = function CardEyebrow({
	as: Component = "p",
	decorate = false,
	className,
	children,
	...props
}) {
	return (
		<Component
			className={clsx(
				className,
				"relative z-10 order-first mb-3 flex items-center text-sm text-cream-400 dark:text-charcole-500",
				decorate && "pl-3.5",
			)}
			{...props}
		>
			{decorate && (
				<span
					className="absolute inset-y-0 left-0 flex items-center"
					aria-hidden="true"
				>
					<span className="h-4 w-0.5 rounded-full bg-cream-200 dark:bg-cream-500" />
				</span>
			)}
			{children}
		</Component>
	);
};
