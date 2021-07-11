<?php require_once  __SITE_PATH . "/view/_header.php";?>

<script src="https://ajax.googleapis.com/ajax/libs/jquery/3.3.1/jquery.js"></script>
<?php $_SESSION["game"]->draw_board() ?>
<br>
<div id="chat"></div>
<br />
<input type="text" id="txt"><button id="btn">Pošalji</button>
<br>

<div id="ranking"></div>
<div id="timer"></div>

<form action=" <?php echo __SITE_URL ?>/index.php" method="get">
    <input type="submit" value="Back"> 
</form>

<script src= <?php echo __SITE_URL . "/js/robot_movement.js"; ?>></script>
<script src= <?php echo __SITE_URL . "/js/gameplay.js"; ?>></script>
<script src= <?php echo __SITE_URL . "/js/utils.js"; ?>></script>

<?php require_once __SITE_PATH . "/view/_footer.php";?>