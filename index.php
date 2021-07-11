<?php
define( '__SITE_PATH', realpath( dirname( __FILE__ ) ) );
define( '__SITE_URL', dirname( $_SERVER['PHP_SELF'] ) );


require_once 'app/init.php';

session_start();

// echo preg_match("#^register/confirmation&sequence=[a-z]{20}$#", $_GET['rt'] . "&sequence=" . $_GET['sequence']); 

// echo "<br> " . $_GET['rt'] . "&sequence=" . $_GET['sequence']." <br>";
if (isset( $_GET["rt"] ) && (isset($_SESSION['player']) || $_GET["rt"] === "login/verifyLogin" 
    || $_GET["rt"] === "register" || $_GET["rt"] === "register/newUser" || 
    ( isset($_GET['sequence']) && preg_match("/^register\/confirmation&sequence=[a-z]{20}$/", $_GET['rt'] . "&sequence=" . $_GET['sequence']) ) ))
    $route = $_GET["rt"];
else 
    $route = "login";

$parts = explode('/', $route);

$controllerName = $parts[0] . "Controller";

if( isset( $parts[1] ) )
	$action = $parts[1];
else
	$action = 'index';

$controllerFileName = __SITE_PATH . "/controller/" . $controllerName . ".php";

require_once $controllerFileName;

$con = new $controllerName;

$con->$action();
?>