use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed,
    LitInt, Type, parse_quote,
};

/// Derives `bitcram::Packable<B>` for a struct or enum.
///
/// The buffer type is supplied as the macro argument:
///
/// ```ignore
/// #[packable(u32)]
/// struct Foo { x: X, y: Y }
/// ```
///
/// All field types must implement `Packable<B>` for the same buffer type `B`,
/// unless the field is annotated with `#[bits(N)]`.
///
/// # `#[bits(N)]` attribute
///
/// Annotates a field with an explicit bit width, allowing types that do not
/// implement `Packable<B>` (typically primitive integers) to be packed:
///
/// ```ignore
/// #[packable(u16)]
/// struct Foo {
///     #[bits(5)] x: u8,
///     #[bits(6)] y: u8,
///     #[bits(5)] z: u8,
/// }
/// ```
///
/// The field type must be cast-compatible with the buffer type in both
/// directions (`*field as B` and `B as FieldType`). All primitive unsigned
/// integers (`u8`, `u16`, `u32`, `u64`, `u128`) qualify.
///
/// # Layout
///
/// Fields are packed in declaration order. The first field occupies the
/// highest bits and the last field occupies the lowest bits.
///
/// For enums, a `ceil(log2(N))`-bit variant index is packed in the lowest
/// bits (or 0 bits if `N == 1`), followed by the variant payload in the
/// higher bits. The size is `variant_index_size + max(variant_payload_sizes)`.
///
/// # Generics
///
/// Type parameters automatically receive a `Packable<B>` bound.
///
/// # Empty enums
///
/// Enums with no variants are silently skipped — no `Packable` impl is
/// generated. This makes the macro non-disruptive during incremental
/// development.
#[proc_macro_attribute]
pub fn packable(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match packable_impl(args, input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn packable_impl(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> syn::Result<TokenStream> {
    let mut input: DeriveInput = syn::parse(input)?;
    let buffer_type: Type = syn::parse(args)?;

    let info = match &input.data {
        Data::Struct(data) => Some(data_struct(data, &buffer_type)?),
        Data::Enum(data) => data_enum(data, &buffer_type)?,
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "Bitcram can only be derived for structs and enums",
            ));
        }
    };

    strip_bits_attrs(&mut input);

    let Some((pack, unpack, size)) = info else {
        return Ok(quote! { #input });
    };

    let ident = &input.ident;
    let mut generics = input.generics.clone();
    let where_clause = generics.make_where_clause();
    for type_param in input.generics.type_params() {
        let type_ident = &type_param.ident;
        where_clause
            .predicates
            .push(parse_quote! { #type_ident: ::bitcram::Packable<#buffer_type> });
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #input
        impl #impl_generics ::bitcram::Packable<#buffer_type> for #ident #ty_generics #where_clause {
            const SIZE: u32 = #size;
            #[inline]
            fn pack(&self) -> #buffer_type {
                #pack
            }
            #[inline]
            fn unpack(buffer: #buffer_type) -> Self {
                #unpack
            }
        }
    })
}

fn data_struct(
    data: &DataStruct,
    buffer_type: &Type,
) -> syn::Result<(TokenStream, TokenStream, TokenStream)> {
    let FieldsInfo {
        pack,
        unpack,
        bracketed_idents,
        size,
    } = match &data.fields {
        Fields::Named(fields) => fields_named(buffer_type, fields)?,
        Fields::Unnamed(fields) => fields_unnamed(buffer_type, fields)?,
        Fields::Unit => {
            return Ok((
                quote! { <#buffer_type as ::bitcram::Buffer>::ZERO },
                quote! { Self },
                quote! { 0 },
            ));
        }
    };

    Ok((
        quote! {
            let mut packer = ::bitcram::Packer::<#buffer_type>::new();
            let Self #bracketed_idents = self;
            #pack
            packer.into_inner()
        },
        quote! {
            let mut unpacker = ::bitcram::Unpacker::<#buffer_type>::new(buffer);
            #unpack
            Self #bracketed_idents
        },
        size,
    ))
}

fn data_enum(
    data: &DataEnum,
    buffer_type: &Type,
) -> syn::Result<Option<(TokenStream, TokenStream, TokenStream)>> {
    let variant_len = data.variants.len();
    let variant_size = match variant_len {
        0 => return Ok(None),
        1 => 0,
        _ => (variant_len - 1).ilog2() + 1,
    };
    let mut pack_variants = Vec::new();
    let mut unpack_variants = Vec::new();
    let mut each_variant_size = Vec::new();

    for (i, variant) in data.variants.iter().enumerate() {
        let FieldsInfo {
            pack,
            unpack,
            bracketed_idents,
            size,
        } = match &variant.fields {
            Fields::Named(fields) => fields_named(buffer_type, fields)?,
            Fields::Unnamed(fields) => fields_unnamed(buffer_type, fields)?,
            Fields::Unit => FieldsInfo {
                pack: quote! {},
                unpack: quote! {},
                bracketed_idents: quote! {},
                size: quote! { 0 },
            },
        };

        let index = i as u32;
        let ident = &variant.ident;

        pack_variants.push(quote! {
            Self::#ident #bracketed_idents => {
                #pack
                packer.raw_pack(#index as #buffer_type, #variant_size);
            }
        });
        unpack_variants.push(quote! {
            #index => {
                #unpack
                Self::#ident #bracketed_idents
            }
        });
        each_variant_size.push(size);
    }

    let pack = quote! {
        let mut packer = ::bitcram::Packer::<#buffer_type>::new();
        match self {
            #(#pack_variants)*
        }
        packer.into_inner()
    };
    let unpack = quote! {
        let mut unpacker = ::bitcram::Unpacker::<#buffer_type>::new(buffer);
        let variant_index = unpacker.raw_unpack(#variant_size) as u32;
        match variant_index {
            #(#unpack_variants)*
            _ => panic!("Invalid variant index"),
        }
    };
    let size = quote! {
        #variant_size + {
            let mut size = 0u32;
            #(
                let s = #each_variant_size;
                if s > size { size = s; }
            )*
            size
        }
    };

    Ok(Some((pack, unpack, size)))
}

struct FieldsInfo {
    pack: TokenStream,
    unpack: TokenStream,
    bracketed_idents: TokenStream,
    size: TokenStream,
}

fn fields_named(buffer_type: &Type, fields: &FieldsNamed) -> syn::Result<FieldsInfo> {
    let mut idents = Vec::new();
    let mut packs = Vec::new();
    let mut unpacks = Vec::new();
    let mut sizes = Vec::new();
    for field in &fields.named {
        let ident = field.ident.as_ref().unwrap();
        idents.push(quote! { #ident });
        let ty = &field.ty;
        let bits = parse_bits_attr(&field.attrs)?;
        let (pack, unpack, size) = field_codegen(quote! { #ident }, ty, bits, buffer_type);
        packs.push(pack);
        unpacks.push(unpack);
        sizes.push(size);
    }
    let unpacks = unpacks.into_iter().rev();
    Ok(FieldsInfo {
        pack: quote! { #(#packs)* },
        unpack: quote! { #(#unpacks)* },
        bracketed_idents: quote! { { #(#idents),* } },
        size: quote! { 0#(+ #sizes)* },
    })
}

fn fields_unnamed(buffer_type: &Type, fields: &FieldsUnnamed) -> syn::Result<FieldsInfo> {
    let mut idents = Vec::new();
    let mut packs = Vec::new();
    let mut unpacks = Vec::new();
    let mut sizes = Vec::new();
    for (i, field) in fields.unnamed.iter().enumerate() {
        let ident = format_ident!("__bitcram_field{}", i);
        idents.push(quote! { #ident });
        let ty = &field.ty;
        let bits = parse_bits_attr(&field.attrs)?;
        let (pack, unpack, size) = field_codegen(quote! { #ident }, ty, bits, buffer_type);
        packs.push(pack);
        unpacks.push(unpack);
        sizes.push(size);
    }
    let unpacks = unpacks.into_iter().rev();
    Ok(FieldsInfo {
        pack: quote! { #(#packs)* },
        unpack: quote! { #(#unpacks)* },
        bracketed_idents: quote! { (#(#idents),*) },
        size: quote! { 0#(+ #sizes)* },
    })
}

fn field_codegen(
    ident: TokenStream,
    ty: &Type,
    bits: Option<u32>,
    buffer_type: &Type,
) -> (TokenStream, TokenStream, TokenStream) {
    if let Some(n) = bits {
        (
            quote! { packer.raw_pack(*#ident as #buffer_type, #n); },
            quote! { let #ident: #ty = unpacker.raw_unpack(#n) as #ty; },
            quote! { #n },
        )
    } else {
        (
            quote! { packer.pack(#ident); },
            quote! { let #ident: #ty = unpacker.unpack(); },
            quote! { <#ty as ::bitcram::Packable<#buffer_type>>::SIZE },
        )
    }
}

fn parse_bits_attr(attrs: &[Attribute]) -> syn::Result<Option<u32>> {
    let mut result = None;
    for attr in attrs {
        if attr.path().is_ident("bits") {
            if result.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "duplicate `#[bits]` attribute",
                ));
            }
            let lit: LitInt = attr.parse_args()?;
            let n: u32 = lit.base10_parse()?;
            if n == 0 {
                return Err(syn::Error::new_spanned(
                    attr,
                    "`#[bits]` value must be greater than 0",
                ));
            }
            result = Some(n);
        }
    }
    Ok(result)
}

fn strip_bits_attrs(input: &mut DeriveInput) {
    match &mut input.data {
        Data::Struct(s) => strip_fields(&mut s.fields),
        Data::Enum(e) => {
            for variant in &mut e.variants {
                strip_fields(&mut variant.fields);
            }
        }
        Data::Union(_) => {}
    }
}

fn strip_fields(fields: &mut Fields) {
    let iter: Box<dyn Iterator<Item = &mut Field>> = match fields {
        Fields::Named(named) => Box::new(named.named.iter_mut()),
        Fields::Unnamed(unnamed) => Box::new(unnamed.unnamed.iter_mut()),
        Fields::Unit => return,
    };
    for field in iter {
        field.attrs.retain(|a| !a.path().is_ident("bits"));
    }
}
