<?php
define( '__SITE_PATH', realpath( dirname( __FILE__ ) ) );
define( '__SITE_URL', dirname( $_SERVER['PHP_SELF'] ) );


require_once 'app/init.php';

session_start();

if (isset($_GET['rt']) && $_GET['rt'] === 'gameplay'){
    clearstatcache();  //kaze internet da ovo treba ako se mijenja cesto file
    if(filesize("./app/timer.log"))
        $route = "mainMenu";
    else 
        $route = $_GET['rt'];

}
else if (isset( $_GET["rt"] ) && (isset($_SESSION['player']) || preg_match("/^(login|register)(.)*$/", $_GET["rt"]) ) )
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