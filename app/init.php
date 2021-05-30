<?php
require_once __SITE_PATH . '/app/database/' . 'database.class.php';
require_once __SITE_PATH . '/misc/constants.php';
require_once __SITE_PATH . '/model/model.class.php';

spl_autoload_register( function( $class_name ) 
{
    //file names in model have to be with all small letters (without separators) and ending with .class.php
    //NOT ricochetRobots
    //NOT ricochet_robots
    //NOT RicochetRobots
    //YES ricochetrobots

	$filename = strtolower($class_name) . '.class.php';
	$file = __SITE_PATH . '/model/' . $filename;

	if( file_exists($file) === false )
	    return false;

	require_once ($file);
} );

?>
