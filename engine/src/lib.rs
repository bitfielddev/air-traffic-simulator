#![warn(
    clippy::as_ptr_cast_mut,
    clippy::as_underscore,
    clippy::bool_to_int_with_if,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_lossless,
    clippy::cast_possible_wrap,
    clippy::checked_conversions,
    clippy::clear_with_drain,
    clippy::clone_on_ref_ptr,
    clippy::cloned_instead_of_copied,
    clippy::cognitive_complexity,
    clippy::collection_is_never_read,
    clippy::copy_iterator,
    clippy::create_dir,
    clippy::default_trait_access,
    clippy::deref_by_slicing,
    clippy::doc_link_with_quotes,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::empty_line_after_outer_attr,
    clippy::empty_structs_with_brackets,
    clippy::enum_glob_use,
    clippy::equatable_if_let,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::explicit_iter_loop,
    clippy::filetype_is_file,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::fn_to_numeric_cast_any,
    clippy::from_iter_instead_of_collect,
    clippy::future_not_send,
    clippy::get_unwrap,
    clippy::if_not_else,
    clippy::if_then_some_else_none,
    clippy::implicit_hasher,
    clippy::impl_trait_in_params,
    clippy::imprecise_flops,
    clippy::inconsistent_struct_constructor,
    clippy::index_refutable_slice,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::items_after_statements,
    clippy::iter_not_returning_iterator,
    clippy::iter_on_empty_collections,
    clippy::iter_on_single_items,
    clippy::iter_with_drain,
    clippy::large_digit_groups,
    clippy::large_futures,
    clippy::large_stack_arrays,
    clippy::large_types_passed_by_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::manual_assert,
    clippy::manual_clamp,
    clippy::manual_instant_elapsed,
    clippy::manual_let_else,
    clippy::manual_ok_or,
    clippy::manual_string_new,
    clippy::many_single_char_names,
    clippy::map_err_ignore,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::mismatching_type_param_order,
    clippy::missing_assert_message,
    clippy::missing_const_for_fn,
    clippy::missing_enforced_import_renames,
    clippy::multiple_unsafe_ops_per_block,
    clippy::must_use_candidate,
    clippy::mut_mut,
    clippy::naive_bytecount,
    clippy::needless_bitwise_bool,
    clippy::needless_collect,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::needless_pass_by_value,
    clippy::negative_feature_names,
    clippy::non_ascii_literal,
    clippy::non_send_fields_in_send_ty,
    clippy::or_fun_call,
    clippy::range_minus_one,
    clippy::range_plus_one,
    clippy::rc_buffer,
    clippy::redundant_closure_for_method_calls,
    clippy::redundant_else,
    clippy::redundant_feature_names,
    clippy::redundant_pub_crate,
    clippy::ref_option_ref,
    clippy::ref_patterns,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::return_self_not_must_use,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::semicolon_inside_block,
    clippy::separated_literal_suffix,
    clippy::significant_drop_in_scrutinee,
    clippy::significant_drop_tightening,
    clippy::single_match_else,
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::string_slice,
    clippy::struct_excessive_bools,
    clippy::suboptimal_flops,
    clippy::suspicious_operation_groupings,
    clippy::suspicious_xor_used_as_pow,
    clippy::tests_outside_test_module,
    clippy::trailing_empty_array,
    clippy::trait_duplication_in_bounds,
    clippy::transmute_ptr_to_ptr,
    clippy::transmute_undefined_repr,
    clippy::trivial_regex,
    clippy::trivially_copy_pass_by_ref,
    clippy::try_err,
    clippy::type_repetition_in_bounds,
    clippy::unchecked_duration_subtraction,
    clippy::undocumented_unsafe_blocks,
    clippy::unicode_not_nfc,
    clippy::uninlined_format_args,
    clippy::unnecessary_box_returns,
    clippy::unnecessary_join,
    clippy::unnecessary_safety_comment,
    clippy::unnecessary_safety_doc,
    clippy::unnecessary_self_imports,
    clippy::unnecessary_struct_initialization,
    clippy::unneeded_field_pattern,
    clippy::unnested_or_patterns,
    clippy::unreadable_literal,
    clippy::unsafe_derive_deserialize,
    clippy::unused_async,
    clippy::unused_peekable,
    clippy::unused_rounding,
    clippy::unused_self,
    clippy::unwrap_in_result,
    clippy::use_self,
    clippy::useless_let_if_seq,
    clippy::verbose_bit_mask,
    clippy::verbose_file_reads
)]
#![deny(
    clippy::derive_partial_eq_without_eq,
    clippy::match_bool,
    clippy::mem_forget,
    clippy::mutex_atomic,
    clippy::mutex_integer,
    clippy::nonstandard_macro_braces,
    clippy::path_buf_push_overwrite,
    clippy::rc_mutex,
    clippy::wildcard_dependencies
)]

use std::{cmp::Ordering, path::Path, sync::Arc};

use eyre::{eyre, Result};
use glam::{Vec2, Vec3};
use smol_str::SmolStr;

type AirportCode = SmolStr;
type Pos2 = Vec2;
type Pos3 = Vec3;
type Class = SmolStr;
type PlaneModelId = SmolStr;
type WaypointId = SmolStr;

#[derive(Clone, Serialize, Deserialize)]
pub struct Engine {
    world: WorldData,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WorldData {
    classes: Arc<[Class]>,
    airports: Arc<[Airport]>,
    flights: Option<Arc<[Flight]>>,
    planes: Arc<[PlaneModel]>,
    waypoints: Arc<[Waypoint]>,
}
impl WorldData {
    pub fn cmp_class(&self, c1: &Class, c2: &Class) -> Result<Ordering> {
        let pos1 = self
            .classes
            .iter()
            .position(|a| a == c1)
            .ok_or_else(|| eyre!("No class `{c1}`"))?;
        let pos2 = self
            .classes
            .iter()
            .position(|a| a == c2)
            .ok_or_else(|| eyre!("No class `{c2}`"))?;
        Ok(pos1.cmp(&pos2))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Airport {
    name: SmolStr,
    code: AirportCode,
    runways: Arc<[Runway]>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Runway {
    start: Pos2,
    end: Pos2,
    altitude: f32,
    class: Class,
}
impl Runway {
    pub fn len(&self) -> f32 {
        (self.start - self.end).length()
    }
    pub fn start3(&self) -> Pos3 {
        self.start.extend(self.altitude)
    }
    pub fn end3(&self) -> Pos3 {
        self.end.extend(self.altitude)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Flight {
    from: AirportCode,
    to: AirportCode,
    plane: Arc<[PlaneModelId]>,
}
impl Flight {
    pub fn from(&self, e: &Engine) -> Result<&Airport> {
        e.world
            .airports
            .iter()
            .find(|a| a.code == self.from)
            .ok_or_else(|| eyre!("No airport `{}`", self.from))
    }
    pub fn to(&self, e: &Engine) -> Result<&Airport> {
        e.world
            .airports
            .iter()
            .find(|a| a.code == self.to)
            .ok_or_else(|| eyre!("No airport `{}`", self.to))
    }
    pub fn plane(&self, e: &Engine) -> Result<Arc<[&PlaneModel]>> {
        self.plane
            .iter()
            .map(|p| {
                e.world
                    .planes
                    .iter()
                    .find(|m| m.id == *p)
                    .ok_or_else(|| eyre!("No plane model `{p}`"))
            })
            .collect::<Result<_, _>>()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlaneModel {
    id: PlaneModelId,
    name: SmolStr,
    manufacturer: SmolStr,
    class: Class,
    icon: Option<Arc<Path>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Waypoint {
    name: WaypointId,
    pos: Pos2,
}
