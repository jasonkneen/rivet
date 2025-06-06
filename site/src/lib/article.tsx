import fs from "node:fs/promises";
import path from "node:path";

import forestAnderson from "@/authors/forest-anderson/avatar.jpeg";
import nathanFlurry from "@/authors/nathan-flurry/avatar.jpeg";
import nicholasKissel from "@/authors/nicholas-kissel/avatar.jpeg";

export const AUTHORS = {
	"nathan-flurry": {
		name: "Nathan Flurry",
		role: "Co-founder & CTO",
		avatar: nathanFlurry,
		socials: {
			twitter: "https://x.com/NathanFlurry/",
			github: "https://github.com/nathanflurry",
			bluesky: "https://bsky.app/profile/nathanflurry.com",
		},
	},
	"nicholas-kissel": {
		name: "Nicholas Kissel",
		role: "Co-founder & CEO",
		avatar: nicholasKissel,
		socials: {
			twitter: "https://x.com/NicholasKissel",
			github: "https://github.com/nicholaskissel",
			bluesky: "https://bsky.app/profile/nicholaskissel.com",
		},
	},
	"forest-anderson": {
		name: "Forest Anderson",
		role: "Founding Engineer",
		avatar: forestAnderson,
		url: "https://twitter.com/angelonfira",
	},
};

export const CATEGORIES = {
	changelog: {
		name: "Changelog",
	},
	"monthly-update": {
		name: "Monthly Update",
	},
	"launch-week": {
		name: "Launch Week",
	},
	technical: {
		name: "Technical",
	},
	guide: {
		name: "Guide",
	},
	frogs: {
		name: "Frogs",
	},
};

export async function loadArticlesMeta() {
	const dir = path.join(process.cwd(), "src/posts");

	const dirs = await fs.readdir(dir, { recursive: true });
	const files = dirs.filter((file) => file.endsWith("page.mdx"));

	const posts = files.map((file) => file.replace(/\/page\.mdx$/, ""));

	return posts;
}

export async function generateArticlesPageParams() {
	const meta = await loadArticlesMeta();

	return meta.map((slug) => {
		return { slug: [slug] };
	});
}

export async function loadArticles() {
	const meta = await loadArticlesMeta();

	return Promise.all(meta.map(loadArticle));
}

export async function loadArticleImage(slug: string) {
	try {
		return await import(`@/posts/${slug}/image.png`);
	} catch {
		try {
			return await import(`@/posts/${slug}/image.jpg`);
		} catch {
			return await import(`@/posts/${slug}/image.gif`);
		}
	}
}

export async function loadArticle(slug: string) {
	const [{ default: Content, ...article }, { default: image }] =
		await Promise.all([
			import(`@/posts/${slug}/page.mdx`),
			loadArticleImage(slug),
		]);

	const author = AUTHORS[article.author];
	if (!author) {
		throw new Error(
			`Unknown author: ${article.author}, please use one of ${Object.keys(AUTHORS).join(", ")}`,
		);
	}

	const category = CATEGORIES[article.category];
	if (!category) {
		throw new Error(
			`Unknown category: ${article.category}, please use one of ${Object.keys(CATEGORIES).join(", ")}`,
		);
	}

	return {
		slug,
		...article,
		published: new Date(article.published),
		category: { ...category, id: article.category },
		Content,
		author,
		image,
	};
}
