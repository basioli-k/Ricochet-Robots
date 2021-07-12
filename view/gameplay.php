<?php require_once  __SITE_PATH . "/view/_header.php";?>

<script src="https://ajax.googleapis.com/ajax/libs/jquery/3.3.1/jquery.js"></script>
<?php $_SESSION["game"]->draw_board() ?>
<br>
<aside id="chat_ranking">
<h3>Chat</h3>
<div id="chat"></div>
<br />
<h3>Licitation</h3>
<div id="ranking"></div>
<h3>Result</h3>
<div id="result"></div>
<input type="text" id="txt" class="input">
<button id="btn" class="button">Pošalji</button>
<br><br>
<div id="timer"></div>
<br><br>
</aside>
<form action=" <?php echo __SITE_URL ?>/index.php" method="get">
    <input type="submit" value="Back" id="gameplay_button" class="button"> 
</form>


<script src= <?php echo __SITE_URL . "/js/robot_movement.js"; ?>></script>
<script src= <?php echo __SITE_URL . "/js/gameplay.js"; ?>></script>
<script src= <?php echo __SITE_URL . "/js/utils.js"; ?>></script>

<?php require_once __SITE_PATH . "/view/_footer.php";?>