"use client";
import "@/generated/framer/styles.css";
import "@/generated/framer/tokens.css";
import IndexFramer from "@/generated/framer/index";
import { useRouter } from "next/navigation";
import { UnframerProvider } from "unframer";

// This file is only used for `use client` directive

export const FramerIndexPage = () => {
	const router = useRouter();
	return (
		<UnframerProvider navigate={router.push}>
			<IndexFramer.Responsive
				style={{ width: "100%", background: "#000000" }}
				variants={{
					xl: "Desktop",
					md: "Tablet",
					sm: "Phone",
					base: "Phone",
				}}
			/>
		</UnframerProvider>
	);
};
