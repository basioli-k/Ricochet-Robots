<?php require_once  __SITE_PATH . "/view/_header.php";?>

<?php $_SESSION["game"]->draw_board() ?>
<br>
<form action=" <?php echo __SITE_URL ?>/index.php?rt=gameplay" method="post">
    <!-- bolje da biramo klikom na robota na ploci bas -->
    <select name="robot">
        <option value=<?php echo RED; ?> selected> Red </option>
        <option value=<?php echo BLUE; ?>> Blue </option>
        <option value=<?php echo GREEN; ?>> Green </option>
        <option value=<?php echo YELLOW; ?>> Yellow </option>
    </select>
    <br />
    <div id="chat"></div>
    <br />
    <input type="text" id="txt"><button id="btn">Pošalji</button>
    <br>
    <input name = "direction" type="submit" value="left">
    <input name = "direction" type="submit" value="top">
    <input name = "direction" type="submit" value="right">
    <input name = "direction" type="submit" value="bottom">

</form>

<script src= <?php echo __SITE_URL . "/js/gameplay.js"; ?>></script>

<?php require_once __SITE_PATH . "/view/_footer.php";?>