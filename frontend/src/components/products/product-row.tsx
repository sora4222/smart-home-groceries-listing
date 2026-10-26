import { ExternalLink } from "lucide-react";
import type { ReactNode } from "react";

import { ProductImage } from "#/components/products/product-image";
import { ProductPrice } from "#/components/products/product-price";
import { UnitPriceLine } from "#/components/products/unit-price-line";
import { Badge } from "#/components/ui/badge";
import type { StoreProduct } from "#/lib/api";
import type { ImageSize } from "#/lib/display-settings";
import { cn } from "#/lib/utils";

/**
 * One product at one store: its picture, what it is, what it costs, its unit
 * price and any deal, and a link to the product on the store's website.
 *
 * Deliberately plain — no marketing copy — so products are compared on
 * picture, price, size and name. `children` are actions on the product
 * (choosing it, disliking it), shown beside the store link. `notice` is a
 * line under them, such as a dislike warning.
 */
export function ProductRow({
	product,
	quantity,
	storeName,
	children,
	notice,
	imageSize = "md",
	showWasPrice = true,
}: {
	product: StoreProduct;
	quantity: number;
	storeName: string;
	imageSize?: ImageSize;
	showWasPrice?: boolean;
	children?: ReactNode;
	notice?: ReactNode;
}) {
	const title = [product.brand, product.name].filter(Boolean).join(" ");

	return (
		<article
			data-testid="store-product"
			data-product-id={product.product_id}
			aria-label={title}
			className={cn(
				"flex gap-3 py-3",
				// A picture at the store's size gets its own line.
				imageSize === "full" ? "flex-col" : "items-start",
				!product.available && "opacity-60",
			)}
		>
			<ProductImage src={product.image_url} name={title} size={imageSize} />
			<div className="flex min-w-0 flex-1 flex-col gap-1">
				<div className="flex items-start justify-between gap-3">
					<div className="flex min-w-0 flex-col">
						<span className="font-medium">{title}</span>
						{product.package_size ? (
							<span className="text-xs text-muted-foreground">
								{product.package_size}
							</span>
						) : null}
					</div>
					<ProductPrice
						product={product}
						quantity={quantity}
						showWasPrice={showWasPrice}
					/>
				</div>

				<UnitPriceLine product={product} />

				<div className="flex flex-wrap items-center gap-1">
					{!product.available && <Badge variant="outline">Unavailable</Badge>}
					{product.on_special && <Badge variant="secondary">Special</Badge>}
					{product.deals.map((deal) => (
						<Badge key={deal.description} variant="outline">
							{deal.description}
						</Badge>
					))}
					<a
						href={product.url}
						target="_blank"
						rel="noreferrer noopener"
						className="ml-auto inline-flex items-center gap-1 rounded-sm text-xs text-muted-foreground underline-offset-4 hover:underline focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
						aria-label={`View ${title} at ${storeName} (opens in a new tab)`}
					>
						View at {storeName}
						<ExternalLink aria-hidden className="size-3" />
					</a>
					{children}
				</div>
				{notice}
			</div>
		</article>
	);
}
