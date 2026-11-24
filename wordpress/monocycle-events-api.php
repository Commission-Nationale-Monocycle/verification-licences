<?php
/**
 * Plugin Name: Monocycle events API
 * Description: Public read-only endpoint listing Sugar Calendar events with their dates.
 *
 * Used by verification-licences to notify about incoming events.
 * Install as `wp-content/mu-plugins/monocycle-events-api.php` on monocycle.info.
 * Nothing must precede `<?php`, otherwise every REST response is broken.
 *
 * GET /wp-json/monocycle/v1/events?after=YYYY-MM-DD&before=YYYY-MM-DD (both included)
 */

add_action( 'rest_api_init', function () {
	$is_date = fn( $value ) => (bool) preg_match( '/^\d{4}-\d{2}-\d{2}$/', $value );

	register_rest_route( 'monocycle/v1', '/events', [
		'methods'             => 'GET',
		'permission_callback' => '__return_true', // Same data as the public calendar.
		'args'                => [
			'after'  => [ 'required' => true, 'validate_callback' => $is_date ],
			'before' => [ 'required' => true, 'validate_callback' => $is_date ],
		],
		'callback'            => function ( WP_REST_Request $request ) {
			if ( ! function_exists( 'sugar_calendar_get_events' ) ) {
				return new WP_Error( 'sugar_calendar_missing', 'Sugar Calendar is not active.', [ 'status' => 503 ] );
			}

			$events = sugar_calendar_get_events( [
				'start_query' => [
					'after'     => $request['after'] . ' 00:00:00',
					'before'    => $request['before'] . ' 23:59:59',
					'inclusive' => true,
				],
				'orderby'     => 'start',
				'order'       => 'ASC',
				'number'      => 100,
			] );

			$published = array_filter(
				$events,
				fn( $event ) => 'publish' === get_post_status( $event->object_id )
			);

			return array_values( array_map( fn( $event ) => [
				'title'   => html_entity_decode( get_the_title( $event->object_id ), ENT_QUOTES, 'UTF-8' ),
				'link'    => get_permalink( $event->object_id ),
				'start'   => $event->start, // "YYYY-MM-DD HH:MM:SS", as entered in the calendar
				'end'     => $event->end,
				'all_day' => (bool) $event->all_day,
			], $published ) );
		},
	] );
} );
